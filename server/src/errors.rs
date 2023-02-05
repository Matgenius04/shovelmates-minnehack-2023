use std::{borrow::Cow, fmt};

use log::{debug, error, info, trace, warn};

use anyhow::anyhow;
use sled::transaction::{ConflictableTransactionError, TransactionError};
use warp::{
    http,
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
    RequestDoesntExist,
    RequestNotAcceptedByUser,
    Json(serde_json::Error),
    Anyhow(anyhow::Error),
}

impl Error {
    pub fn unexpected<E>(e: E) -> Error
    where
        anyhow::Error: From<E>,
    {
        Error::Anyhow(e.into())
    }

    pub fn msg<M>(m: M) -> Error
    where
        M: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        Error::unexpected(anyhow::Error::msg(m))
    }
}

impl From<Error> for ConflictableTransactionError<Error> {
    fn from(value: Error) -> Self {
        ConflictableTransactionError::Abort(value)
    }
}

impl From<TransactionError<Error>> for Error {
    fn from(value: TransactionError<Error>) -> Self {
        match value {
            TransactionError::Abort(e) => e,
            TransactionError::Storage(e) => Error::Anyhow(anyhow!(e)),
        }
    }
}

impl From<http::Error> for Error {
    fn from(value: http::Error) -> Self {
        Error::unexpected(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::unexpected(value)
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
            RequestDoesntExist => "That request doesn't exist".into(),
            RequestNotAcceptedByUser => "That request wasn't accepted by the user".into(),
            Json(e) => format!("Failed to decode body: {e}").into(),
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
            RequestDoesntExist => StatusCode::CONFLICT,
            RequestNotAcceptedByUser => StatusCode::CONFLICT,
            Json(_) => StatusCode::BAD_REQUEST,
            Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn log(&self) {
        use Error::*;

        match self {
            Anyhow(_) => error!("{}", self.description()),
            InvalidToken | IncorrectPassword(_) => warn!("{}", self.description()),
            NotSenior | NotVolunteer | RequestDoesntExist | RequestNotAcceptedByUser => {
                info!("{}", self.description())
            }
            Json(_) => debug!("{}", self.description()),
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
