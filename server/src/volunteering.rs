use log::{debug, error, trace};
use serde::Deserialize;
use serde_json::json;
use warp::{
    body::bytes,
    hyper::{body::Bytes, Body, Response},
    Filter, Rejection,
};

use crate::{
    authorization::authorize, clone, clone_dbs, db::Archived, distance_meters, errors::Error,
    extract_json, ArchivedUserType, HelpRequestDB, HelpRequestState, InfallibleDeserialize, User,
    UserDB, UserType,
};

fn volunteering_endpoint(
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

    if !matches!(user.user_type, ArchivedUserType::Volunteer(_)) {
        return Err(Error::NotVolunteer);
    }

    callback(bytes, username, user)
}

pub fn volunteering_filters(
    user_db: &UserDB,
    help_requests: &HelpRequestDB,
) -> impl Filter<Extract = (Result<Response<Body>, Error>,), Error = Rejection> + Clone {
    let request_work = warp::path!("api" / "request-work")
        .and(bytes())
        .and(clone_dbs(user_db, help_requests))
        .map(move |bytes, users_db, requests_db| {
            volunteering_endpoint(&bytes, &users_db, |_, username, user| {
                debug!("{username} is requesting work");
                request_work(user, &requests_db, &users_db)
            })
        });

    let get_request = warp::path!("api" / "get-request")
        .and(bytes())
        .and(clone_dbs(user_db, help_requests))
        .map(move |bytes, users_db, requests_db| {
            volunteering_endpoint(&bytes, &users_db, |bytes, username, user| {
                debug!("{username} is getting a request");
                get_request(
                    extract_json::<GetRequestData>(bytes)?.id,
                    user,
                    &users_db,
                    &requests_db,
                )
            })
        });

    let accept_request = warp::path!("api" / "get-request")
        .and(bytes())
        .and(clone_dbs(user_db, help_requests))
        .map(move |bytes, users_db, requests_db| {
            volunteering_endpoint(&bytes, &users_db, |bytes, username, user| {
                debug!("{username} is getting a request");
                accept_request(
                    username,
                    user,
                    extract_json::<GetRequestData>(bytes)?.id,
                    &users_db,
                    &requests_db,
                )
            })
        });

    let accepted_requests = warp::path!("api" / "accepted-requests")
        .and(bytes())
        .and(clone(user_db.to_owned()))
        .map(move |bytes, users_db| {
            volunteering_endpoint(&bytes, &users_db, |_, username, user| {
                debug!("{username} is getting their accepted requests");
                accepted_requests(user)
            })
        });

    let marking_completed = warp::path!("api" / "mark-request-completed")
        .and(bytes())
        .and(clone_dbs(user_db, help_requests))
        .map(move |bytes, users_db, requests_db| {
            volunteering_endpoint(&bytes, &users_db, |bytes, username, _| {
                debug!("{username} is marking a request as completed");
                marking_as_completed(
                    username,
                    extract_json::<GetRequestData>(bytes)?.id,
                    &requests_db,
                )
            })
        });

    warp::post().and(
        request_work
            .or(get_request)
            .unify()
            .or(accepted_requests)
            .unify()
            .or(accept_request)
            .unify()
            .or(marking_completed)
            .unify(),
    )
}

fn request_work(
    user: Archived<User>,
    help_requests: &HelpRequestDB,
    user_db: &UserDB,
) -> Result<Response<Body>, Error> {
    let coords = user.location;

    let mut requests = help_requests
        .iter()
        .map(|maybe_request| {
            let (id, request) = maybe_request?;

            Ok((request.distance_meters(coords, user_db)?, id))
        })
        .filter(|v| match v {
            Ok((dist, _)) => !dist.is_nan(),
            Err(_) => true,
        })
        .collect::<Result<Vec<_>, anyhow::Error>>()?;

    requests.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).expect("NaN values were filtered"));

    Ok(Response::builder()
        .status(200)
        .body(Body::from(serde_json::to_string(
            match requests.get(0..100) {
                Some(v) => v,
                None => &requests,
            },
        )?))?)
}

#[derive(Deserialize)]
struct GetRequestData {
    id: String,
}

fn get_request(
    id: String,
    user: Archived<User>,
    user_db: &UserDB,
    help_requests: &HelpRequestDB,
) -> Result<Response<Body>, Error> {
    Ok(match help_requests.get(&id)? {
        Some(request) => {
            let senior = match user_db.get(&request.username)? {
                Some(v) => v,
                None => {
                    return Err(Error::Anyhow(anyhow::Error::msg(
                        "The senior in the request doesn't exist in the database",
                    )))
                }
            };

            let dist = distance_meters(user.location, senior.location);

            Response::builder()
                .status(200)
                .body(Body::from(serde_json::to_string(&json!({
                    "user": {
                        "username": &*senior.username,
                        "name": &*senior.name,
                    },
                    "picture": &*request.picture,
                    "notes": &*request.notes,
                    "dist": dist,
                    "address": &*senior.address,
                }))?))?
        }
        None => Response::builder()
            .status(409)
            .body(Body::from("That request doesn't exist"))?,
    })
}

fn accept_request(
    username: String,
    user: Archived<User>,
    id: String,
    user_db: &UserDB,
    help_requests: &HelpRequestDB,
) -> Result<Response<Body>, Error> {
    let mut user_de = user.to_original();

    let mut accepted = match user_de.user_type {
        UserType::Volunteer(accepted) => accepted,
        _ => {
            return Err(Error::Anyhow(anyhow::Error::msg(
                "The user isn't a volunteer, this case should've been filtered earlier",
            )))
        }
    };

    help_requests.update::<rkyv::Infallible, ()>(&id, move |t| {
        t.state = HelpRequestState::AcceptedBy(username.to_owned());
    })?;

    accepted.push(id);

    user_de.user_type = UserType::Volunteer(accepted);

    user_db.add(&user.username, &user_de)?;

    Ok(Response::builder().status(200).body(Body::empty())?)
}

fn accepted_requests(user: Archived<User>) -> Result<Response<Body>, Error> {
    match &user.user_type {
        ArchivedUserType::Volunteer(accepted) => {
            Ok(Response::builder()
                .status(200)
                .body(Body::from(serde_json::to_string::<Vec<String>>(
                    &accepted.deserialize(),
                )?))?)
        }
        _ => Err(Error::Anyhow(anyhow::Error::msg(
            "The user isn't a volunteer, this case should've been filtered earlier",
        ))),
    }
}

fn marking_as_completed(
    username: String,
    id: String,
    help_requests: &HelpRequestDB,
) -> Result<Response<Body>, Error> {
    Ok(
        match help_requests.update::<rkyv::Infallible, _>(&id, |request| {
            if let HelpRequestState::AcceptedBy(accepted_by) = &request.state {
                if &username == accepted_by {
                    request.state = HelpRequestState::MarkedCompletedBy(username.to_owned());

                    return Response::builder().status(200).body(Body::empty());
                }
            }

            return Response::builder()
                .status(409)
                .body(Body::from("The id wasn't accepted by the user"));
        })? {
            Some(v) => v?,
            None => Response::builder()
                .status(409)
                .body(Body::from("The id doesn't exist"))?,
        },
    )
}
