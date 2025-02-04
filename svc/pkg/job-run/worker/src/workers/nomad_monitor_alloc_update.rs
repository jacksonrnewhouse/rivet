use chirp_worker::prelude::*;
use proto::backend::pkg::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AllocationUpdated {
	allocation: nomad_client::models::Allocation,
}

#[derive(Debug, Copy, Clone)]
enum TaskState {
	Pending,
	Running,
	Dead,
}

#[worker(name = "job-run-nomad-monitor-alloc-update")]
async fn worker(
	ctx: OperationContext<job_run::msg::nomad_monitor_alloc_update::Message>,
) -> GlobalResult<()> {
	let crdb = ctx.crdb("db-job-state").await?;

	let AllocationUpdated { allocation: alloc } = serde_json::from_str(&ctx.payload_json)?;
	let alloc_state_json = serde_json::to_value(&alloc)?;

	let alloc_id = internal_unwrap!(alloc.ID);
	let eval_id = internal_unwrap!(alloc.eval_id, "alloc has no eval");
	let job_id = internal_unwrap!(alloc.job_id);
	let client_status = internal_unwrap!(alloc.client_status);
	let task_states = internal_unwrap!(alloc.task_states);

	if !util_job::is_nomad_job_run(job_id) {
		tracing::info!(%job_id, "disregarding event");
		return Ok(());
	}

	// Get the main task by finding the task that is not the run cleanup task
	let main_task = task_states
		.iter()
		.filter(|(k, _)| k.as_str() != util_job::RUN_CLEANUP_TASK_NAME)
		.map(|(_, v)| v)
		.next();
	let main_task = internal_unwrap_owned!(main_task, "could not find main task");
	let main_task_state_raw = internal_unwrap!(main_task.state);
	let main_task_events = internal_unwrap!(main_task.events);

	tracing::info!(
		?client_status,
		?alloc_id,
		?eval_id,
		?job_id,
		?main_task_state_raw,
		?main_task_events,
		"alloc updated"
	);

	let main_task_state = match main_task_state_raw.as_str() {
		"pending" => TaskState::Pending,
		"running" => TaskState::Running,
		"dead" => TaskState::Dead,
		_ => {
			tracing::error!(?main_task_state_raw, "unknown task state");
			return Ok(());
		}
	};

	match main_task_state {
		TaskState::Pending => {
			tracing::info!("run pending");

			let run_row = sqlx::query_as::<_, (Uuid,)>(indoc!(
				"
				UPDATE run_meta_nomad
				SET alloc_state = $2
				WHERE dispatched_job_id = $1
				RETURNING run_id
				"
			))
			.bind(job_id)
			.bind(&alloc_state_json)
			.fetch_optional(&crdb)
			.await?;

			if run_row.is_none() {
				if ctx.req_dt() > util::duration::minutes(5) {
					tracing::error!("discarding stale message");
					return Ok(());
				} else {
					retry_panic!("run not found, may be race condition with insertion");
				}
			};

			Ok(())
		}
		TaskState::Running => {
			let run_row = sqlx::query_as::<_, (Uuid, Option<i64>)>(indoc!(
				"
				WITH
					select_run AS (
						SELECT runs.run_id, runs.start_ts
						FROM run_meta_nomad
						INNER JOIN runs ON runs.run_id = run_meta_nomad.run_id
						WHERE dispatched_job_id = $1
					),
					_update_runs AS (
						UPDATE runs
						SET start_ts = $2
						FROM select_run
						WHERE
							runs.run_id = select_run.run_id AND
							runs.start_ts IS NULL
						RETURNING 1
					),
					_update_run_meta_nomad AS (
						UPDATE run_meta_nomad
						SET alloc_state = $3
						FROM select_run
						WHERE run_meta_nomad.run_id = select_run.run_id
						RETURNING 1
					)
				SELECT * FROM select_run
				"
			))
			.bind(job_id)
			.bind(ctx.ts())
			.bind(&alloc_state_json)
			.fetch_optional(&crdb)
			.await?;

			let Some((run_id, start_ts)) = run_row else {
				if ctx.req_dt() > util::duration::minutes(5) {
					tracing::error!("discarding stale message");
					return Ok(());
				} else {
					retry_panic!("run not found, may be race condition with insertion");
				}
			};

			if start_ts.is_none() {
				tracing::info!("run started");

				msg!([ctx] job_run::msg::started(run_id) {
					run_id: Some(run_id.into()),
				})
				.await?;

				Ok(())
			} else {
				tracing::info!("run already started");

				Ok(())
			}
		}
		TaskState::Dead => {
			let run_row = sqlx::query_as::<_, (Uuid, Option<i64>)>(indoc!(
				"
				WITH
					select_run AS (
						SELECT runs.run_id, runs.finish_ts
						FROM run_meta_nomad
						INNER JOIN runs ON runs.run_id = run_meta_nomad.run_id
						WHERE dispatched_job_id = $1
					),
					_update_runs AS (
						UPDATE runs
						SET finish_ts = $2
						FROM select_run
						WHERE
							runs.run_id = select_run.run_id AND
							runs.finish_ts IS NULL
						RETURNING 1
					),
					_update_run_meta_nomad AS (
						UPDATE run_meta_nomad
						SET alloc_state = $3
						FROM select_run
						WHERE run_meta_nomad.run_id = select_run.run_id
						RETURNING 1
					)
				SELECT * FROM select_run
				"
			))
			.bind(job_id)
			.bind(ctx.ts())
			.bind(&alloc_state_json)
			.fetch_optional(&crdb)
			.await?;

			let Some((run_id, finish_ts)) = run_row else {
				if ctx.req_dt() > util::duration::minutes(5) {
					tracing::error!("discarding stale message");
					return Ok(());
				} else {
					retry_panic!("run not found, may be race condition with insertion");
				}
			};

			if finish_ts.is_none() {
				tracing::info!("run finished");

				// Publish message
				//
				// It's fine if this is called multiple times. The operation is
				// idempotent and it's better to ensure the job gets cleaned up
				// rather than forgotten.
				msg!([ctx] job_run::msg::cleanup(run_id) {
					run_id: Some(run_id.into()),
					..Default::default()
				})
				.await?;
				msg!([ctx] job_run::msg::finished(run_id) {
					run_id: Some(run_id.into()),
				})
				.await?;

				Ok(())
			} else {
				tracing::info!("run already finished");
				Ok(())
			}
		}
	}
}
