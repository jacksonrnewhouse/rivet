use proto::backend;
use rivet_operation::prelude::*;
use rivet_party_server::models;

pub fn handle_without_presence(
	current_user_id: &Uuid,
	user: &backend::user::User,
) -> GlobalResult<models::IdentityHandle> {
	let user_id = internal_unwrap!(user.user_id).as_uuid();
	let is_self = &user_id == current_user_id;

	Ok(models::IdentityHandle {
		identity_id: user_id.to_string(),
		display_name: user.display_name.to_owned(),
		account_number: user.account_number as i32,
		avatar_url: util::route::user_avatar(
			&user.avatar_id,
			user.profile_upload_id.as_ref().map(common::Uuid::as_uuid),
			user.profile_file_name.as_ref(),
		),
		presence: None,
		party: None,
		is_registered: true, // TODO:
		external: models::IdentityExternalLinks {
			profile: util::route::user_profile(&user_id),
			settings: None,
			chat: (!is_self).then(|| util::route::user_chat(&user_id)),
		},
	})
}
