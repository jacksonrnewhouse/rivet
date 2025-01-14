use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;

#[derive(sqlx::FromRow)]
struct GameVersion {
	version_id: Uuid,
}

#[operation(name = "kv-config-version-get")]
async fn handle(
	ctx: OperationContext<kv_config::version_get::Request>,
) -> GlobalResult<kv_config::version_get::Response> {
	let version_ids = ctx
		.version_ids
		.iter()
		.map(common::Uuid::as_uuid)
		.collect::<Vec<_>>();

	let versions = sqlx::query_as::<_, GameVersion>(indoc!(
		"
			SELECT version_id
			FROM game_versions
			WHERE version_id = ANY($1)
		"
	))
	.bind(version_ids)
	.fetch_all(&ctx.crdb("db-kv-config").await?)
	.await?
	.into_iter()
	.map(|version| kv_config::version_get::response::Version {
		version_id: Some(version.version_id.into()),
		config: Some(backend::kv::VersionConfig {}),
		config_meta: Some(backend::kv::VersionConfigMeta {}),
	})
	.collect::<Vec<_>>();

	Ok(kv_config::version_get::Response { versions })
}
