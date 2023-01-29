use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use warp::{hyper::Response, reject, Filter, Rejection};

use crate::{
    authorization::authorize, db::Db, rejections::CustomRejection, HelpRequest, HelpRequestState,
    User, UserType,
};

fn help_requests_initial_validation(
    user_db: &Db<str, User>,
) -> impl Filter<Extract = (String, User), Error = Rejection> + Clone {
    let invoker_getter_db = user_db.to_owned();

    authorize()
        .and_then(move |username: String| {
            let user_db = invoker_getter_db.to_owned();
            async move {
                let user = user_db
                    .get(&username)
                    .map_err(|e| reject::custom::<CustomRejection>(e.into()))?
                    .ok_or_else(|| reject::custom(CustomRejection::InvalidToken))?;

                if !matches!(user.user_type, UserType::Senior(_)) {
                    return Err(reject::custom(CustomRejection::NotSenior));
                }

                Ok((username, user))
            }
        })
        .untuple_one()
}

pub fn help_requests_filters(
    user_db: &Db<str, User>,
    help_requests: &Db<[u8; 32], HelpRequest>,
) -> impl Filter<Extract = (Response<String>,), Error = Rejection> + Clone {
    let request_help_requests_db = help_requests.to_owned();
    let request_help_users_db = user_db.to_owned();
    let request_help = warp::path!("api" / "request-help")
        .and(help_requests_initial_validation(user_db))
        .and(warp::body::json::<RequestHelpInfo>())
        .and_then(move |_, user, request_help_info| {
            let requests_db = request_help_requests_db.to_owned();
            let users_db = request_help_users_db.to_owned();
            async move {
                request_help(user, request_help_info, &requests_db, &users_db)
                    .map_err(reject::custom)
            }
        });

    let get_request_requests_db = help_requests.to_owned();
    let get_requests = warp::path!("api" / "help-requests")
        .and(help_requests_initial_validation(user_db))
        .and_then(move |_, user| {
            let requests_db = get_request_requests_db.to_owned();
            async move { get_help_request(user, &requests_db).map_err(reject::custom) }
        });

    warp::post().and(request_help.or(get_requests).unify())
}

#[derive(Deserialize)]
struct RequestHelpInfo {
    picture: String,
    notes: String,
}

fn request_help(
    mut user: User,
    request_help_info: RequestHelpInfo,
    help_requests: &Db<[u8; 32], HelpRequest>,
    users: &Db<str, User>,
) -> Result<Response<String>, CustomRejection> {
    if let UserType::Senior(Some(_)) = &user.user_type {
        return Err(CustomRejection::AlreadyRequestedHelp);
    }

    let help_request = HelpRequest {
        picture: request_help_info.picture,
        notes: request_help_info.notes,
        creation_time: Utc::now().timestamp_millis(),
        state: HelpRequestState::Pending,
        username: user.username,
    };

    let mut id;

    loop {
        id = rand::random();

        if !help_requests.contains(&id)? {
            break;
        }
    }

    help_requests.add(&id, &help_request)?;

    // Transfer ownership back
    user.username = help_request.username;

    user.user_type = UserType::Senior(Some(id));

    users.add(&user.username, &user)?;

    Ok(Response::builder().status(200).body(String::new())?)
}

fn get_help_request(
    user: User,
    help_requests: &Db<[u8; 32], HelpRequest>,
) -> Result<Response<String>, CustomRejection> {
    if let UserType::Senior(Some(id)) = user.user_type {
        let help_request = match help_requests.get(&id)? {
            Some(v) => v,
            None => return Err(CustomRejection::Anyhow(anyhow::Error::msg(
                "The ID for the help request stored in the server doesn't exist in the database",
            ))),
        };

        Ok(Response::builder()
            .status(200)
            .body(serde_json::to_string(&json!({
                "picture": help_request.picture,
                "notes": help_request.notes,
                "creationTime": help_request.creation_time,
                "state": help_request.state,
            }))?)?)
    } else {
        Err(CustomRejection::DidntRequestHelp)
    }
}
