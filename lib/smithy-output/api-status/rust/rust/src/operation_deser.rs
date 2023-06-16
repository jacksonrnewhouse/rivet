// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn parse_matchmaker_error(
	response: &http::Response<bytes::Bytes>,
) -> std::result::Result<crate::output::MatchmakerOutput, crate::error::MatchmakerError> {
	let generic = crate::json_deser::parse_http_generic_error(response)
		.map_err(crate::error::MatchmakerError::unhandled)?;
	let error_code = match generic.code() {
		Some(code) => code,
		None => return Err(crate::error::MatchmakerError::unhandled(generic)),
	};

	let _error_message = generic.message().map(|msg| msg.to_owned());
	Err(match error_code {
		"InternalError" => crate::error::MatchmakerError {
			meta: generic,
			kind: crate::error::MatchmakerErrorKind::InternalError({
				#[allow(unused_mut)]
				let mut tmp = {
					#[allow(unused_mut)]
					let mut output = crate::error::internal_error::Builder::default();
					let _ = response;
					output =
						crate::json_deser::deser_structure_crate_error_internal_error_json_err(
							response.body().as_ref(),
							output,
						)
						.map_err(crate::error::MatchmakerError::unhandled)?;
					output.build()
				};
				if (&tmp.message).is_none() {
					tmp.message = _error_message;
				}
				tmp
			}),
		},
		"RateLimitError" => {
			crate::error::MatchmakerError {
				meta: generic,
				kind: crate::error::MatchmakerErrorKind::RateLimitError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::rate_limit_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_rate_limit_error_json_err(response.body().as_ref(), output).map_err(crate::error::MatchmakerError::unhandled)?;
						output.build()
					};
					if (&tmp.message).is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"ForbiddenError" => {
			crate::error::MatchmakerError {
				meta: generic,
				kind: crate::error::MatchmakerErrorKind::ForbiddenError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::forbidden_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_forbidden_error_json_err(response.body().as_ref(), output).map_err(crate::error::MatchmakerError::unhandled)?;
						output.build()
					};
					if (&tmp.message).is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"UnauthorizedError" => {
			crate::error::MatchmakerError {
				meta: generic,
				kind: crate::error::MatchmakerErrorKind::UnauthorizedError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::unauthorized_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_unauthorized_error_json_err(response.body().as_ref(), output).map_err(crate::error::MatchmakerError::unhandled)?;
						output.build()
					};
					if (&tmp.message).is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"NotFoundError" => {
			crate::error::MatchmakerError {
				meta: generic,
				kind: crate::error::MatchmakerErrorKind::NotFoundError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::not_found_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_not_found_error_json_err(response.body().as_ref(), output).map_err(crate::error::MatchmakerError::unhandled)?;
						output.build()
					};
					if (&tmp.message).is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		"BadRequestError" => {
			crate::error::MatchmakerError {
				meta: generic,
				kind: crate::error::MatchmakerErrorKind::BadRequestError({
					#[allow(unused_mut)]
					let mut tmp = {
						#[allow(unused_mut)]
						let mut output = crate::error::bad_request_error::Builder::default();
						let _ = response;
						output = crate::json_deser::deser_structure_crate_error_bad_request_error_json_err(response.body().as_ref(), output).map_err(crate::error::MatchmakerError::unhandled)?;
						output.build()
					};
					if (&tmp.message).is_none() {
						tmp.message = _error_message;
					}
					tmp
				}),
			}
		}
		_ => crate::error::MatchmakerError::generic(generic),
	})
}

#[allow(clippy::unnecessary_wraps)]
pub fn parse_matchmaker_response(
	response: &http::Response<bytes::Bytes>,
) -> std::result::Result<crate::output::MatchmakerOutput, crate::error::MatchmakerError> {
	Ok({
		#[allow(unused_mut)]
		let mut output = crate::output::matchmaker_output::Builder::default();
		let _ = response;
		output.build()
	})
}