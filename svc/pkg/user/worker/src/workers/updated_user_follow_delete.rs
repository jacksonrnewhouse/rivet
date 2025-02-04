use chirp_worker::prelude::*;
use proto::backend::{self, pkg::*};

#[worker(name = "user-updated-user-follow-delete")]
async fn worker(ctx: OperationContext<user_follow::msg::delete::Message>) -> GlobalResult<()> {
	let follower_user_id = internal_unwrap!(ctx.follower_user_id);
	let following_user_id = internal_unwrap!(ctx.following_user_id);

	msg!([ctx] user::msg::updated(following_user_id) {
		user_id: ctx.following_user_id,
		update: Some(backend::user::update::Update {
			kind: Some(backend::user::update::update::Kind::FollowDelete(backend::user::update::FollowDelete {
				follower_user_id: ctx.follower_user_id,
			})),
		}),
	})
	.await?;

	msg!([ctx] user::msg::updated(follower_user_id) {
		user_id: ctx.follower_user_id,
		update: Some(backend::user::update::Update {
			kind: Some(backend::user::update::update::Kind::FollowDelete(backend::user::update::FollowDelete {
				follower_user_id: ctx.following_user_id,
			})),
		}),
	})
	.await?;

	Ok(())
}
