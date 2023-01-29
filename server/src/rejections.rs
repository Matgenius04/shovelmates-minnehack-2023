use log::{debug, error, info, trace, warn};

use warp::{
    body::BodyDeserializeError,
    hyper::{Response, StatusCode},
    reject, Rejection, Reply,
};

#[derive(Debug)]
pub enum CustomRejection {
    InvalidToken,
    UsernameAlreadyExists(String),
    UsernameDoesntExist(String),
    IncorrectPassword(String),
    NotSenior,
    NotVolunteer,
    AlreadyRequestedHelp,
    DidntRequestHelp,
    Anyhow(anyhow::Error),
}

impl<T> From<T> for CustomRejection
where
    anyhow::Error: From<T>,
{
    fn from(val: T) -> CustomRejection {
        CustomRejection::Anyhow(anyhow::Error::from(val))
    }
}

impl CustomRejection {
    fn description(&self) -> String {
        use CustomRejection::*;

        match self {
            InvalidToken => "The authentication string was invalid".to_owned(),
            UsernameAlreadyExists(username) => format!("The username `{username}` already exists"),
            UsernameDoesntExist(username) => format!("The username `{username}` doesn't exist"),
            IncorrectPassword(username) => format!("The password for `{username} is incorrect`"),
            NotSenior => {
                "You must have a Senior account to invoke help request endpoints".to_owned()
            }
            NotVolunteer => {
                "You must have a Volunteer account to invoke volunteering endpoints".to_owned()
            }
            AlreadyRequestedHelp => "You already requested help".to_owned(),
            DidntRequestHelp => "You never requested help".to_owned(),
            Anyhow(e) => format!("Unexpected server error: {e}"),
        }
    }

    fn status(&self) -> StatusCode {
        use CustomRejection::*;

        match self {
            InvalidToken => StatusCode::FORBIDDEN,
            UsernameAlreadyExists(_) => StatusCode::CONFLICT,
            UsernameDoesntExist(_) => StatusCode::CONFLICT,
            IncorrectPassword(_) => StatusCode::FORBIDDEN,
            NotSenior => StatusCode::METHOD_NOT_ALLOWED,
            NotVolunteer => StatusCode::METHOD_NOT_ALLOWED,
            AlreadyRequestedHelp => StatusCode::CONFLICT,
            DidntRequestHelp => StatusCode::CONFLICT,
            Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn log(&self) {
        use CustomRejection::*;

        match self {
            Anyhow(_) => error!("{}", self.description()),
            InvalidToken | IncorrectPassword(_) => warn!("{}", self.description()),
            NotSenior | NotVolunteer => info!("{}", self.description()),
            UsernameAlreadyExists(_)
            | UsernameDoesntExist(_)
            | AlreadyRequestedHelp
            | DidntRequestHelp => {
                trace!("{}", self.description())
            }
        }
    }
}

impl reject::Reject for CustomRejection {}

pub async fn handle_rejection(rejection: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(e) = rejection.find::<BodyDeserializeError>() {
        debug!("Body deserialze error: {e}");
        return Response::builder()
            .status(400)
            .body("Error deserializing body".to_owned())
            .map_err(|e| reject::custom(CustomRejection::from(e)));
    }

    if let Some(custom_rejection) = rejection.find::<CustomRejection>() {
        custom_rejection.log();

        return Ok(match custom_rejection {
            e => Response::builder()
                .status(e.status())
                .body(e.description())
                .map_err(|e| reject::custom(CustomRejection::from(e)))?,
        });
    }

    Err(rejection)
}
