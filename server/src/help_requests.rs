use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use chrono::Utc;
use log::{debug, error, info, trace};
use rkyv::option::ArchivedOption;
use serde::Deserialize;
use serde_json::json;
use warp::{
    body::bytes,
    hyper::{body::Bytes, Body},
    Filter, Rejection,
};

use crate::{
    authorization::authorize,
    clone_dbs,
    db::{Archived, Transactional},
    errors::Error,
    extract_json, ArchivedUserType, HelpRequest, HelpRequestDB, HelpRequestState, User, UserDB,
    UserType,
};

fn help_request_endpoint(
    bytes: &Bytes,
    user_db: &UserDB,
    callback: impl FnOnce(&Bytes, String, Archived<User>) -> Result<Body, Error>,
) -> Result<Body, Error> {
    trace!("Validating request for a help requests endpoint");

    let username = authorize(bytes)?;

    let user = user_db
        .get(&username)?
        .ok_or_else(|| {error!("A token with an incorrect username was generated or someone cracked the tokens somehow"); Error::msg("Oofy token")})?;

    if !matches!(user.user_type, ArchivedUserType::Senior(_)) {
        return Err(Error::NotSenior);
    }

    callback(bytes, username, user)
}

pub fn help_requests_filters(
    user_db: &UserDB,
    help_requests: &HelpRequestDB,
) -> impl Filter<Extract = (Result<Body, Error>,), Error = Rejection> + Clone {
    let request_help = warp::path!("api" / "request-help")
        .and(bytes())
        .and(clone_dbs(user_db, help_requests))
        .map(move |bytes, users_db, requests_db| request_help(&bytes, &users_db, &requests_db));

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
            delete_help_request(&bytes, &users_db, &requests_db)
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
    bytes: &Bytes,
    users: &UserDB,
    help_requests: &HelpRequestDB,
) -> Result<Body, Error> {
    let username = authorize(bytes)?;
    let request_help_info = extract_json::<RequestHelpInfo>(bytes)?;

    info!("{username} is requesting help");

    (users, help_requests)
        .transaction(move |(users_db, requests_db)| {
            let user = match users_db.get(&username)? {
                Some(v) => v,
                None => {
                    return Err(
                        Error::msg("There exists a token for a user that doesn't exist").into(),
                    )
                }
            };

            if let ArchivedUserType::Senior(ArchivedOption::Some(_)) = &user.user_type {
                return Err(Error::AlreadyRequestedHelp.into());
            }

            let mut user_de: User = user.to_original();

            let help_request = HelpRequest {
                picture: request_help_info.picture.to_owned(),
                notes: request_help_info.notes.to_owned(),
                creation_time: Utc::now().timestamp_millis(),
                state: HelpRequestState::Pending,
                username: user_de.username,
            };

            let id = URL_SAFE.encode(requests_db.generate_id()?.to_le_bytes());

            requests_db.add(&id, &help_request)?;

            // Transfer ownership back
            user_de.username = help_request.username;

            user_de.user_type = UserType::Senior(Some(id));

            users_db.add(&user.username, &user_de)?;

            info!(
                "`{}` successfully created a request for help",
                user.username
            );

            Ok(Body::from("{}"))
        })
        .map_err(|e| e.into())
}

fn get_help_request(user: Archived<User>, help_requests: &HelpRequestDB) -> Result<Body, Error> {
    if let ArchivedUserType::Senior(ArchivedOption::Some(id)) = &user.user_type {
        let help_request = match help_requests.get(id)? {
            Some(v) => v,
            None => return Err(Error::msg(
                "The ID for the help request stored in the server doesn't exist in the database",
            )),
        };

        debug!(
            "`{}` successfully retrieved their help request",
            user.username
        );

        Ok(Body::from(serde_json::to_string(&json!({
            "picture": &*help_request.picture,
            "notes": &*help_request.notes,
            "creationTime": help_request.creation_time,
            "state": help_request.state.to_json(),
        }))?))
    } else {
        Err(Error::DidntRequestHelp)
    }
}

fn delete_help_request(
    bytes: &Bytes,
    user_db: &UserDB,
    help_requests: &HelpRequestDB,
) -> Result<Body, Error> {
    let username = authorize(bytes)?;

    (user_db, help_requests)
        .transaction(|(users_db, requests_db)| {
            debug!("`{username}` hit delete-help-request endpoint");

            let user = match users_db.get(&username)? {
                Some(v) => v,
                None => {
                    return Err(Error::msg(
                        "There exists a token for a username that doesn't exist",
                    )
                    .into())
                }
            };

            match &user.user_type {
                ArchivedUserType::Senior(maybe_id) => match maybe_id {
                    ArchivedOption::Some(id) => {
                        if requests_db.delete(id)?.is_none() {
                            return Err(Error::msg(
                                "The ID for the help request stored in the server doesn't exist in the database",
                            ).into());
                        };

                        let mut user_de = user.to_original();

                        user_de.user_type = UserType::Senior(None);

                        users_db.add(&user.username, &user_de)?;

                        info!(
                            "`{}` successfully deleted their help request",
                            user.username
                        );

                        Ok(Body::from("Successfully deleted help request"))
                    }
                    ArchivedOption::None => {
                        debug!(
                            "`{}` tried to delete their help request but there was nothing to delete",
                            user.username
                        );

                        Ok(Body::from("There was nothing to delete"))
                    }
                },
                _ => Err(Error::NotSenior.into()),
            }
        })
        .map_err(|e| e.into())
}
