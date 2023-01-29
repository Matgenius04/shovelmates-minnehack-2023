use log::{debug, trace};
use serde::Deserialize;
use serde_json::json;
use warp::{hyper::Response, reject, Filter, Rejection};

use crate::{
    authorization::authorize, db::Db, distance_meters, rejections::CustomRejection, HelpRequest,
    User, UserType,
};

fn volunteering_initial_validation(
    user_db: &Db<User>,
) -> impl Filter<Extract = (String, User), Error = Rejection> + Clone {
    trace!("Validating request for a help requests endpoint");

    let invoker_getter_db = user_db.to_owned();

    authorize()
        .and_then(move |username: String| {
            let user_db = invoker_getter_db.to_owned();
            async move {
                let user = user_db
                    .get(&username)
                    .map_err(|e| reject::custom::<CustomRejection>(e.into()))?
                    .ok_or_else(|| reject::custom(CustomRejection::InvalidToken))?;

                if !matches!(user.user_type, UserType::Volunteer(_)) {
                    return Err(reject::custom(CustomRejection::NotVolunteer));
                }

                Ok((username, user))
            }
        })
        .untuple_one()
}

pub fn volunteering_filters(
    user_db: &Db<User>,
    help_requests: &Db<HelpRequest>,
) -> impl Filter<Extract = (Response<String>,), Error = Rejection> + Clone {
    let request_work_requests_db = help_requests.to_owned();
    let request_work_user_db = user_db.to_owned();
    let request_work = warp::path!("api" / "request-work")
        .and(volunteering_initial_validation(user_db))
        .and_then(move |username, user| {
            debug!("{username} is requesting work");
            let requests_db = request_work_requests_db.to_owned();
            let user_db = request_work_user_db.to_owned();
            async move { request_work(user, &requests_db, &user_db).map_err(reject::custom) }
        });

    let get_request_requests_db = help_requests.to_owned();
    let get_request_user_db = user_db.to_owned();
    let get_request = warp::path!("api" / "request-work")
        .and(volunteering_initial_validation(user_db))
        .and(warp::filters::body::json::<GetRequestData>())
        .and_then(move |username, user, get_request_data: GetRequestData| {
            debug!("{username} is requesting work");
            let requests_db = get_request_requests_db.to_owned();
            let user_db = get_request_user_db.to_owned();
            async move {
                get_request(get_request_data.id, user, &user_db, &requests_db)
                    .map_err(reject::custom)
            }
        });

    warp::post().and(request_work.or(get_request).unify())
}

fn request_work(
    user: User,
    help_requests: &Db<HelpRequest>,
    user_db: &Db<User>,
) -> Result<Response<String>, CustomRejection> {
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

    Ok(Response::builder().status(200).body(serde_json::to_string(
        match requests.get(0..100) {
            Some(v) => v,
            None => &requests,
        },
    )?)?)
}

#[derive(Deserialize)]
struct GetRequestData {
    id: String,
}

fn get_request(
    id: String,
    user: User,
    user_db: &Db<User>,
    help_requests: &Db<HelpRequest>,
) -> Result<Response<String>, CustomRejection> {
    Ok(match help_requests.get(&id)? {
        Some(request) => {
            let senior = match user_db.get(&request.username)? {
                Some(v) => v,
                None => {
                    return Err(CustomRejection::Anyhow(anyhow::Error::msg(
                        "The senior in the request doesn't exist in the database",
                    )))
                }
            };

            let dist = distance_meters(user.location, senior.location);

            Response::builder()
                .status(200)
                .body(serde_json::to_string(&json!({
                    "user": {
                        "username": senior.username,
                        "name": senior.name,
                    },
                    "picture": request.picture,
                    "notes": request.notes,
                    "dist": dist,
                    "address": senior.address,
                }))?)?
        }
        None => Response::builder()
            .status(409)
            .body("That request doesn't exist".to_string())?,
    })
}
