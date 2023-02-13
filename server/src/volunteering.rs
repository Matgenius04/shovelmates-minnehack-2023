use log::{debug, error, trace};
use serde::Deserialize;
use serde_json::json;

use warp::{
    body::bytes,
    hyper::{body::Bytes, Body},
    Filter, Rejection,
};

use crate::{
    authorization::authorize,
    clone, clone_dbs,
    db::{Archived, Transactional},
    distance_meters,
    errors::Error,
    extract_json, ArchivedUserType, HelpRequestDB, HelpRequestState, InfallibleDeserialize, User,
    UserDB, UserType,
};

fn volunteering_endpoint(
    bytes: &Bytes,
    user_db: &UserDB,
    callback: impl FnOnce(&Bytes, String, Archived<User>) -> Result<Body, Error>,
) -> Result<Body, Error> {
    trace!("Validating request for a help requests endpoint");

    let username = authorize(bytes)?;

    let user = user_db
        .get(&username)?
        .ok_or_else(|| {error!("A token with an incorrect username was generated or someone cracked the tokens somehow"); Error::msg("Oofy token")})?;

    if !matches!(user.user_type, ArchivedUserType::Volunteer(_)) {
        return Err(Error::NotVolunteer);
    }

    callback(bytes, username, user)
}

pub fn volunteering_filters(
    user_db: &UserDB,
    help_requests: &HelpRequestDB,
) -> impl Filter<Extract = (Result<Body, Error>,), Error = Rejection> + Clone {
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

    let accept_request = warp::path!("api" / "accept-request")
        .and(bytes())
        .and(clone_dbs(user_db, help_requests))
        .map(move |bytes, users_db, requests_db| accept_request(&bytes, &users_db, &requests_db));

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
) -> Result<Body, Error> {
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
        .collect::<Result<Vec<_>, Error>>()?;

    requests.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).expect("NaN values were filtered"));

    Ok(Body::from(serde_json::to_string(
        match requests.get(0..100) {
            Some(v) => v,
            None => &requests,
        },
    )?))
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
) -> Result<Body, Error> {
    match help_requests.get(&id)? {
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

            Ok(Body::from(serde_json::to_string(&json!({
                "user": {
                    "username": &*senior.username,
                    "name": &*senior.name,
                },
                "picture": &*request.picture,
                "notes": &*request.notes,
                "dist": dist,
                "address": &*senior.address,
            }))?))
        }
        None => Err(Error::RequestDoesntExist),
    }
}

fn accept_request(
    bytes: &Bytes,
    user_db: &UserDB,
    help_requests: &HelpRequestDB,
) -> Result<Body, Error> {
    let username = authorize(bytes)?;

    let id = extract_json::<GetRequestData>(bytes)?.id;

    debug!("{username} is accepting a request");

    (user_db, help_requests)
        .transaction(move |(user_db, requests_db)| {
            let mut user = user_db
                .get(&username)?
                .ok_or_else(|| Error::msg("There exists a token for a user that doesn't exist"))?
                .to_original();

            let mut accepted = match user.user_type {
                UserType::Volunteer(accepted) => accepted,
                _ => return Err(Error::NotVolunteer.into()),
            };

            let mut help_request = match requests_db.get(&id)? {
                Some(v) => v,
                None => return Err(Error::RequestDoesntExist.into()),
            }
            .to_original();

            help_request.state = HelpRequestState::AcceptedBy(username.to_owned());

            requests_db.add(&id, &help_request)?;

            accepted.push(id.to_owned());

            user.user_type = UserType::Volunteer(accepted);

            user_db.add(&username, &user)?;

            Ok(Body::from("{}"))
        })
        .map_err(|e| e.into())
}

fn accepted_requests(user: Archived<User>) -> Result<Body, Error> {
    match &user.user_type {
        ArchivedUserType::Volunteer(accepted) => Ok(Body::from(serde_json::to_string::<
            Vec<String>,
        >(&accepted.deserialize())?)),
        _ => Err(Error::Anyhow(anyhow::Error::msg(
            "The user isn't a volunteer, this case should've been filtered earlier",
        ))),
    }
}

fn marking_as_completed(
    username: String,
    id: String,
    help_requests: &HelpRequestDB,
) -> Result<Body, Error> {
    help_requests
        .transaction(|requests_db| {
            let mut request = match requests_db.get(&id)? {
                Some(v) => v.to_original(),
                None => return Err(Error::RequestDoesntExist.into()),
            };

            match request.state {
                HelpRequestState::AcceptedBy(accepted_by) if username == accepted_by => {
                    request.state = HelpRequestState::MarkedCompletedBy(accepted_by);
                }
                _ => return Err(Error::RequestNotAcceptedByUser.into()),
            }

            requests_db.add(&id, &request)?;

            Ok(Body::from("{}"))
        })
        .map_err(|e| e.into())
}
