use futures_util::stream::{StreamExt, TryStreamExt};
use proto::backend::pkg::*;
use rivet_operation::prelude::*;
use serde_json::json;
use std::{collections::HashMap, time::Duration};

#[derive(Debug, sqlx::FromRow)]
struct FileRow {
	path: String,
	content_length: i64,
	nsfw_score_threshold: Option<f32>,
}

#[operation(name = "upload-complete")]
async fn handle(
	ctx: OperationContext<upload::complete::Request>,
) -> GlobalResult<upload::complete::Response> {
	let crdb = ctx.crdb("db-upload").await?;

	let upload_id = internal_unwrap!(ctx.upload_id).as_uuid();

	let (bucket, files, user_id) = fetch_files(&crdb, upload_id).await?;
	let files_len = files.len();

	if let Some(req_bucket) = &ctx.bucket {
		assert_eq_with!(&bucket, req_bucket, DB_INVALID_BUCKET);
	}

	let s3_client = s3_util::Client::from_env(&bucket).await?;

	let nsfw_scores =
		validate_profanity_scores(&ctx, &s3_client, upload_id, &files, user_id).await?;

	validate_files(&s3_client, upload_id, files).await?;

	// Mark as complete
	sqlx::query(indoc!(
		"
		UPDATE uploads
		SET complete_ts = $2
		WHERE upload_id = $1
		"
	))
	.bind(upload_id)
	.bind(ctx.ts())
	.execute(&crdb)
	.await?;

	msg!([ctx] upload::msg::complete_complete(upload_id) {
		upload_id: Some(upload_id.into()),
	})
	.await?;

	let analytics_nsfw_scores = nsfw_scores.map(|scores| {
		json!({
			"min": scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.min(b)),
			"max": scores.iter().fold(f32::INFINITY, |a, &b| a.max(b)),
			"mean": scores.iter().sum::<f32>() / scores.len() as f32,
		})
	});
	msg!([ctx] analytics::msg::event_create() {
		events: vec![
			analytics::msg::event_create::Event {
				name: "upload.complete".into(),
				properties_json: Some(serde_json::to_string(&json!({
					"user_id": user_id,
					"upload_id": upload_id,
					"bucket": bucket,
					"files_len": files_len,
					"nsfw_scores": analytics_nsfw_scores,
				}))?),
				..Default::default()
			}
		],
	})
	.await?;

	Ok(upload::complete::Response {})
}

async fn fetch_files(
	crdb: &CrdbPool,
	upload_id: Uuid,
) -> GlobalResult<(String, Vec<FileRow>, Option<Uuid>)> {
	let (bucket, user_id) = sqlx::query_as::<_, (String, Option<Uuid>)>(indoc!(
		"
		SELECT bucket, user_id
		FROM uploads
		WHERE upload_id = $1
		"
	))
	.bind(upload_id)
	.fetch_one(crdb)
	.await?;

	let files = sqlx::query_as::<_, FileRow>(indoc!(
		"
		SELECT path, content_length, nsfw_score_threshold
		FROM upload_files
		WHERE upload_id = $1
		"
	))
	.bind(upload_id)
	.fetch_all(crdb)
	.await?;

	tracing::info!(?bucket, files_len = ?files.len(), "fetched files");

	Ok((bucket, files, user_id))
}

async fn validate_profanity_scores(
	ctx: &OperationContext<upload::complete::Request>,
	s3_client: &s3_util::Client,
	upload_id: Uuid,
	files: &[FileRow],
	user_id: Option<Uuid>,
) -> GlobalResult<Option<Vec<f32>>> {
	tracing::info!("validating profanity scores");

	// Validate profanity scores
	let nsfw_required_scores = futures_util::stream::iter(files)
		// Filter out files that don't need to match a profanity score
		.filter_map(|file_row| async move {
			file_row
				.nsfw_score_threshold
				.map(|x| (format!("{}/{}", upload_id, file_row.path), x))
		})
		// Generate presigned get requests for the profanity filter to fetch
		.then(|(key, score)| async move {
			let presigned_req = s3_client
				.get_object()
				.bucket(s3_client.bucket())
				.key(key)
				.presigned(
					s3_util::aws_sdk_s3::presigning::config::PresigningConfig::builder()
						.expires_in(std::time::Duration::from_secs(5 * 60))
						.build()?,
				)
				.await?;
			let url = presigned_req.uri().to_string();
			GlobalResult::Ok((url, score))
		})
		.try_collect::<HashMap<String, f32>>()
		.await?;

	let scores = if !nsfw_required_scores.is_empty() {
		// Score teh images
		let score_res = op!([ctx] nsfw_image_score {
			image_urls: nsfw_required_scores.keys().cloned().collect(),
		})
		.await?;

		// Validate the images fall within the approved scores
		for score in &score_res.scores {
			let required_score = internal_unwrap_owned!(nsfw_required_scores.get(&score.url));
			if score.score >= *required_score {
				msg!([ctx] analytics::msg::event_create() {
					events: vec![
						analytics::msg::event_create::Event {
							name: "upload.nsfw_detected".into(),
							properties_json: Some(serde_json::to_string(&json!({
								"user_id": user_id,
								"upload_id": upload_id,
								"bucket": s3_client.bucket(),
								"url": score.url,
								"required_score": required_score,
								"score": score.score,
							}))?),
							..Default::default()
						}
					],
				})
				.await?;

				if ctx.test()
					|| std::env::var("RIVET_UPLOAD_NSFW_ERROR_VERBSOE")
						.ok()
						.map_or(false, |x| x == "1")
				{
					panic_with!(UPLOAD_NSFW_CONTENT_DETECTED {
						metadata: serde_json::json!({
							"url": score.url,
							"score": score.score,
						}),
					});
				} else {
					// Don't expose the score in production to prevent
					// exploitation
					panic_with!(UPLOAD_NSFW_CONTENT_DETECTED);
				}
			}
		}

		let scores = score_res.scores.iter().map(|x| x.score).collect::<Vec<_>>();

		Some(scores)
	} else {
		None
	};

	Ok(scores)
}

async fn validate_files(
	s3_client: &s3_util::Client,
	upload_id: Uuid,
	files: Vec<FileRow>,
) -> GlobalResult<()> {
	tracing::info!("validating files");

	let files_len = files.len();
	futures_util::stream::iter(files.into_iter().enumerate())
		.map(|(i, file_row)| async move {
			// Fetch & validate file metadata
			let mut fail_idx = 0;
			let head_obj = loop {
				let head_obj_res = s3_client
					.head_object()
					.bucket(s3_client.bucket())
					.key(format!("{}/{}", upload_id, file_row.path))
					.send()
					.await;
				match head_obj_res {
					Ok(x) => break x,
					Err(err) => {
						fail_idx += 1;

						if fail_idx > 4 {
							tracing::error!(?fail_idx, "head object failed too many times");
							return Err(err.into());
						} else {
							tracing::warn!(?fail_idx, "head object failed, retrying due to likely benign error from backblaze with malformed last-modified header");
							tokio::time::sleep(Duration::from_millis(500)).await;
						}
					}
				}
			};

			// This should never be triggered since we use prepared uploads, but
			// we validate it regardless
			internal_assert_eq!(
				file_row.content_length,
				head_obj.content_length,
				"incorrect content length"
			);

			if i % 1000 == 0 {
				tracing::info!("fetched file metadata ({i}/{files_len})")
			}

			GlobalResult::Ok(())
		})
		.buffer_unordered(32)
		.try_collect::<Vec<_>>()
		.await?;

	Ok(())
}
