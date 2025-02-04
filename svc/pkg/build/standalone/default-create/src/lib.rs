use indoc::indoc;
use proto::backend;
use rivet_operation::prelude::*;
use uuid::Uuid;

const CONTENT_TYPE: &str = "application/x-tar";

const DEFAULT_BUILDS: &[DefaultBuildConfig] = &[
	DefaultBuildConfig {
		kind: "game-multiplayer",
		tag: include_str!("../default-builds/outputs/game-multiplayer-tag.txt"),
		tar: include_bytes!("../default-builds/outputs/game-multiplayer.tar"),
	},
	DefaultBuildConfig {
		kind: "test-fail-immediately",
		tag: include_str!("../default-builds/outputs/test-fail-immediately-tag.txt"),
		tar: include_bytes!("../default-builds/outputs/test-fail-immediately.tar"),
	},
	DefaultBuildConfig {
		kind: "test-hang-indefinitely",
		tag: include_str!("../default-builds/outputs/test-hang-indefinitely-tag.txt"),
		tar: include_bytes!("../default-builds/outputs/test-hang-indefinitely.tar"),
	},
	DefaultBuildConfig {
		kind: "test-mm-lobby-ready",
		tag: include_str!("../default-builds/outputs/test-mm-lobby-ready-tag.txt"),
		tar: include_bytes!("../default-builds/outputs/test-mm-lobby-ready.tar"),
	},
];

struct DefaultBuildConfig {
	/// The kind of default build.
	kind: &'static str,
	/// Tag for the image that's archived.
	tag: &'static str,
	/// Bytes for the image that needs to be uploaded.
	tar: &'static [u8],
}

#[tracing::instrument]
pub async fn run_from_env() -> GlobalResult<()> {
	let pools = rivet_pools::from_env("build-default-create").await?;
	let client =
		chirp_client::SharedClient::from_env(pools.clone())?.wrap_new("build-default-create");
	let cache = rivet_cache::CacheInner::from_env(pools.clone())?;
	let ctx = OperationContext::new(
		"build-default-create".into(),
		std::time::Duration::from_secs(60),
		rivet_connection::Connection::new(client, pools, cache),
		Uuid::new_v4(),
		Uuid::new_v4(),
		util::timestamp::now(),
		util::timestamp::now(),
		(),
		Vec::new(),
	);
	let crdb_pool = ctx.crdb("db-build").await?;

	for build in DEFAULT_BUILDS {
		// Check if this default build is already set
		let old_default_build =
			sqlx::query_as::<_, (String,)>("SELECT image_tag FROM default_builds WHERE kind = $1")
				.bind(build.kind)
				.fetch_optional(&crdb_pool)
				.await?;
		if old_default_build
			.as_ref()
			.map_or(false, |(old_image_tag,)| old_image_tag == build.tag)
		{
			tracing::info!(
				?old_default_build,
				"build already matches the given tag, skipping"
			);
			return Ok(());
		}

		// Upload the build
		tracing::info!(tag = %build.tag, "uploading new build");
		let upload_id = upload_build(&ctx, build).await?;

		// Update default build
		tracing::info!(tag = %build.tag, ?upload_id, "setting default build");
		sqlx::query(indoc!(
			"
			UPSERT INTO default_builds (kind, image_tag, upload_id)
			VALUES ($1, $2, $3)
			"
		))
		.bind(build.kind)
		.bind(build.tag)
		.bind(upload_id)
		.execute(&crdb_pool)
		.await?;
	}

	Ok(())
}

async fn upload_build(
	ctx: &OperationContext<()>,
	build: &DefaultBuildConfig,
) -> GlobalResult<Uuid> {
	let upload_prepare_res = op!([ctx] upload_prepare {
		bucket: "bucket-build".into(),
		files: vec![
			backend::upload::PrepareFile {
				path: "image.tar".into(),
				mime: Some(CONTENT_TYPE.into()),
				content_length: build.tar.len() as u64,
				..Default::default()
			},
		],
	})
	.await?;
	let upload_id = internal_unwrap!(upload_prepare_res.upload_id).as_uuid();
	let req = internal_unwrap_owned!(upload_prepare_res.presigned_requests.first());

	let url = &req.url;
	tracing::info!(%url, "uploading file");
	let res = reqwest::Client::new()
		.put(url)
		.header(reqwest::header::CONTENT_TYPE, CONTENT_TYPE)
		.header(reqwest::header::CONTENT_LENGTH, build.tar.len() as u64)
		.body(reqwest::Body::from(build.tar))
		.send()
		.await?;
	if res.status().is_success() {
		tracing::info!("successfully uploaded");
	} else {
		tracing::warn!(status = ?res.status(), "failure uploading");
	}

	// Complete the upload
	op!([ctx] upload_complete {
		upload_id: Some(upload_id.into()),
		bucket: Some("bucket-build".into()),
	})
	.await?;

	Ok(upload_id)
}
