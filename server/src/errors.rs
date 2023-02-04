use std::borrow::Cow;

use log::{debug, error, info, trace, warn};

use warp::{
    hyper::{Body, Response, StatusCode},
    Reply,
};

#[derive(Debug)]
pub enum Error {
    InvalidToken,
    UsernameAlreadyExists(String),
    UsernameDoesntExist(String),
    IncorrectPassword(String),
    NotSenior,
    NotVolunteer,
    AlreadyRequestedHelp,
    DidntRequestHelp,
    JSON(serde_json::Error),
    Anyhow(anyhow::Error),
}

impl<T> From<T> for Error
where
    anyhow::Error: From<T>,
{
    fn from(val: T) -> Error {
        Error::Anyhow(anyhow::Error::from(val))
    }
}

impl Error {
    fn description(&self) -> Cow<'static, str> {
        use Error::*;

        match self {
            InvalidToken => "The authentication string was invalid".into(),
            UsernameAlreadyExists(username) => {
                format!("The username `{username}` already exists").into()
            }
            UsernameDoesntExist(username) => {
                format!("The username `{username}` doesn't exist").into()
            }
            IncorrectPassword(username) => {
                format!("The password for `{username} is incorrect`").into()
            }
            NotSenior => "You must have a Senior account to invoke help request endpoints".into(),
            NotVolunteer => {
                "You must have a Volunteer account to invoke volunteering endpoints".into()
            }
            AlreadyRequestedHelp => "You already requested help".into(),
            DidntRequestHelp => "You never requested help".into(),
            JSON(e) => format!("Failed to decode body: {e}").into(),
            Anyhow(e) => format!("Unexpected server error: {e}").into(),
        }
    }

    fn status(&self) -> StatusCode {
        use Error::*;

        match self {
            InvalidToken => StatusCode::FORBIDDEN,
            UsernameAlreadyExists(_) => StatusCode::CONFLICT,
            UsernameDoesntExist(_) => StatusCode::CONFLICT,
            IncorrectPassword(_) => StatusCode::FORBIDDEN,
            NotSenior => StatusCode::METHOD_NOT_ALLOWED,
            NotVolunteer => StatusCode::METHOD_NOT_ALLOWED,
            AlreadyRequestedHelp => StatusCode::CONFLICT,
            DidntRequestHelp => StatusCode::CONFLICT,
            JSON(_) => StatusCode::BAD_REQUEST,
            Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn log(&self) {
        use Error::*;

        match self {
            Anyhow(_) => error!("{}", self.description()),
            InvalidToken | IncorrectPassword(_) => warn!("{}", self.description()),
            NotSenior | NotVolunteer => info!("{}", self.description()),
            JSON(_) => debug!("{}", self.description()),
            UsernameAlreadyExists(_)
            | UsernameDoesntExist(_)
            | AlreadyRequestedHelp
            | DidntRequestHelp => {
                trace!("{}", self.description())
            }
        }
    }

    pub fn into_response(self) -> Response<Body> {
        self.log();

        match Response::builder()
            .status(self.status())
            .body(Body::from(self.description()))
        {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to create response: {e}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
