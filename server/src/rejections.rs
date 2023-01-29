use log::{error, info, trace, warn};
use thiserror::Error;
use warp::{
    body::BodyDeserializeError,
    http,
    hyper::{Response, StatusCode},
    reject, Rejection, Reply,
};

#[derive(Debug, Error)]
pub enum CustomRejection {
    #[error("Failed to create a response: {0}")]
    ResponseCreationError(#[from] http::Error),
    #[error("The authentication string was invalid")]
    InvalidToken,
    #[error("The username `{0}` already exists")]
    UsernameAlreadyExists(String),
    #[error("The username `{0}` doesn't exist")]
    UsernameDoesntExist(String),
    #[error("The password for `{0}` is incorrect")]
    IncorrectPassword(String),
    #[error("You must have a Senior account to invoke help request endpoints")]
    NotSenior,
    #[error("You must have a Volunteer account to invoke help request endpoints")]
    NotVolunteer,
    #[error("You already requested help")]
    AlreadyRequestedHelp,
    #[error("Unexpected server error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl CustomRejection {
    fn status(&self) -> StatusCode {
        use CustomRejection::*;

        match self {
            ResponseCreationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            InvalidToken => StatusCode::FORBIDDEN,
            UsernameAlreadyExists(_) => StatusCode::CONFLICT,
            UsernameDoesntExist(_) => StatusCode::CONFLICT,
            IncorrectPassword(_) => StatusCode::FORBIDDEN,
            NotSenior => StatusCode::METHOD_NOT_ALLOWED,
            NotVolunteer => StatusCode::METHOD_NOT_ALLOWED,
            AlreadyRequestedHelp => StatusCode::CONFLICT,
            Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn log(&self) {
        use CustomRejection::*;

        match self {
            ResponseCreationError(_) | Anyhow(_) => error!("{}", self.to_string()),
            InvalidToken | IncorrectPassword(_) => warn!("{}", self.to_string()),
            NotSenior | NotVolunteer => info!("{}", self.to_string()),
            UsernameAlreadyExists(_) | UsernameDoesntExist(_) | AlreadyRequestedHelp => {
                trace!("{}", self.to_string())
            }
        }
    }
}

impl reject::Reject for CustomRejection {}

pub async fn handle_rejection(rejection: Rejection) -> Result<impl Reply, Rejection> {
    if rejection.find::<BodyDeserializeError>().is_some() {
        return Response::builder()
            .status(400)
            .body("Error deserializing body".to_owned())
            .map_err(|e| reject::custom(CustomRejection::from(e)));
    }

    if let Some(custom_rejection) = rejection.find::<CustomRejection>() {
        use CustomRejection::*;

        custom_rejection.log();

        return Ok(match custom_rejection {
            ResponseCreationError(_) => return Err(rejection),
            e => Response::builder()
                .status(e.status())
                .body(e.to_string())
                .map_err(|e| reject::custom(CustomRejection::from(e)))?,
        });
    }

    Err(rejection)
}
