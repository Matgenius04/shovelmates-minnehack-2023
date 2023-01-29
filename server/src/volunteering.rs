use log::{debug, trace};
use warp::{hyper::Response, reject, Filter, Rejection};

use crate::{
    authorization::authorize, db::Db, rejections::CustomRejection, HelpRequest, User, UserType,
};

fn volunteering_initial_validation(
    user_db: &Db<str, User>,
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
    user_db: &Db<str, User>,
    help_requests: &Db<[u8; 32], HelpRequest>,
) -> impl Filter<Extract = (Response<String>,), Error = Rejection> + Clone {
    let request_work_requests_db = help_requests.to_owned();
    let request_work = warp::path!("api" / "request-work")
        .and(volunteering_initial_validation(user_db))
        .and_then(move |username, user| {
            debug!("{username} is requesting work");
            let requests_db = request_work_requests_db.to_owned();
            async move { request_work(user, &requests_db).map_err(reject::custom) }
        });

    warp::post().and(request_work)
}

fn request_work(
    user: User,
    help_requests: &Db<[u8; 32], HelpRequest>,
) -> Result<Response<String>, CustomRejection> {
    todo!()
}
