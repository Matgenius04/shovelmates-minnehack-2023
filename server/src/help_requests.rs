use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use chrono::Utc;
use log::{debug, error, info, trace};
use rkyv::option::ArchivedOption;
use serde::Deserialize;
use serde_json::json;
use warp::{
    body::bytes,
    hyper::{body::Bytes, Body, Response},
    Filter, Rejection,
};

use crate::{
    authorization::authorize, clone_dbs, db::Archived, errors::Error, extract_json,
    ArchivedUserType, HelpRequest, HelpRequestDB, HelpRequestState, User, UserDB, UserType,
};

fn help_request_endpoint(
    bytes: &Bytes,
    user_db: &UserDB,
    callback: impl FnOnce(&Bytes, String, Archived<User>) -> Result<Response<Body>, Error>,
) -> Result<Response<Body>, Error> {
    trace!("Validating request for a help requests endpoint");

    let invoker_getter_db = user_db.to_owned();

    let username = authorize(bytes)?;
    let user_db = invoker_getter_db.to_owned();

    let user = user_db
        .get(&username)?
        .ok_or_else(|| {error!("A token with an incorrect username was generated or someone cracked the tokens somehow"); anyhow::Error::msg("Oofy token")})?;

    if !matches!(user.user_type, ArchivedUserType::Senior(_)) {
        return Err(Error::NotSenior);
    }

    callback(bytes, username, user)
}

pub fn help_requests_filters(
    user_db: &UserDB,
    help_requests: &HelpRequestDB,
) -> impl Filter<Extract = (Result<Response<Body>, Error>,), Error = Rejection> + Clone {
    let request_help = warp::path!("api" / "request-help")
        .and(bytes())
        .and(clone_dbs(user_db, help_requests))
        .map(move |bytes, users_db, requests_db| {
            help_request_endpoint(&bytes, &users_db, |bytes, username, user| {
                debug!("`{username}` hit request-help endpoint");
                request_help(
                    user,
                    extract_json::<RequestHelpInfo>(bytes)?,
                    &requests_db,
                    &users_db,
                )
            })
        });

    let get_requests = warp::path!("api" / "help-requests")
        .and(bytes())
        .and(clone_dbs(user_db, help_requests))
        .map(move |bytes, users_db, requests_db| {
            help_request_endpoint(&bytes, &users_db, |_, username, user| {
                debug!("`{username}` hit help-requests endpoint");
                get_help_request(user, &requests_db)
            })
        });

    let delete_request = warp::path!("api" / "delete-help-request")
        .and(bytes())
        .and(clone_dbs(user_db, help_requests))
        .map(move |bytes, users_db, requests_db| {
            help_request_endpoint(&bytes, &users_db, |_, username, user| {
                debug!("`{username}` hit delete-help-request endpoint");
                delete_help_request(user, &users_db, &requests_db)
            })
        });

    warp::post().and(
        request_help
            .or(get_requests)
            .unify()
            .or(delete_request)
            .unify(),
    )
}

#[derive(Deserialize)]
struct RequestHelpInfo {
    picture: String,
    notes: String,
}

fn request_help(
    user: Archived<User>,
    request_help_info: RequestHelpInfo,
    help_requests: &HelpRequestDB,
    users: &UserDB,
) -> Result<Response<Body>, Error> {
    if let ArchivedUserType::Senior(ArchivedOption::Some(_)) = &user.user_type {
        return Err(Error::AlreadyRequestedHelp);
    }

    let mut user_de: User = user.to_original();

    let help_request = HelpRequest {
        picture: request_help_info.picture,
        notes: request_help_info.notes,
        creation_time: Utc::now().timestamp_millis(),
        state: HelpRequestState::Pending,
        username: user_de.username,
    };

    let mut id;

    loop {
        id = URL_SAFE.encode(rand::random::<[u8; 32]>());

        if !help_requests.contains(&id)? {
            break;
        }
    }

    help_requests.add(&id, &help_request)?;

    // Transfer ownership back
    user_de.username = help_request.username;

    user_de.user_type = UserType::Senior(Some(id));

    users.add(&user.username, &user_de)?;

    info!(
        "`{}` successfully created a request for help",
        user.username
    );

    Ok(Response::builder().status(200).body(Body::empty())?)
}

fn get_help_request(
    user: Archived<User>,
    help_requests: &HelpRequestDB,
) -> Result<Response<Body>, Error> {
    if let ArchivedUserType::Senior(ArchivedOption::Some(id)) = &user.user_type {
        let help_request = match help_requests.get(&id)? {
            Some(v) => v,
            None => return Err(Error::Anyhow(anyhow::Error::msg(
                "The ID for the help request stored in the server doesn't exist in the database",
            ))),
        };

        debug!(
            "`{}` successfully retrieved their help request",
            user.username
        );

        Ok(Response::builder()
            .status(200)
            .body(Body::from(serde_json::to_string(&json!({
                "picture": &*help_request.picture,
                "notes": &*help_request.notes,
                "creationTime": help_request.creation_time,
                "state": help_request.state.to_json(),
            }))?))?)
    } else {
        Err(Error::DidntRequestHelp)
    }
}

fn delete_help_request(
    user: Archived<User>,
    user_db: &UserDB,
    help_requests: &HelpRequestDB,
) -> Result<Response<Body>, Error> {
    if let ArchivedUserType::Senior(ArchivedOption::Some(id)) = &user.user_type {
        if help_requests.delete(&id)?.is_none() {
            return Err(Error::Anyhow(anyhow::Error::msg(
                "The ID for the help request stored in the server doesn't exist in the database",
            )));
        };

        let mut user_de = user.to_original();

        user_de.user_type = UserType::Senior(None);

        user_db.add(&user.username, &user_de)?;

        info!(
            "`{}` successfully deleted their help request",
            user.username
        );

        return Ok(Response::builder()
            .status(200)
            .body(Body::from("Successfully deleted help request"))?);
    }

    debug!(
        "`{}` tried to delete their help request but there was nothing to delete",
        user.username
    );

    Ok(Response::builder()
        .status(200)
        .body(Body::from("There was nothing to delete"))?)
}
