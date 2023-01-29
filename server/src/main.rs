mod accounts;
mod authorization;
mod db;
mod help_requests;
mod rejections;
mod volunteering;

use db::Db;
use log::info;
use serde::{Deserialize, Serialize};
use warp::Filter;

use crate::{
    accounts::accounts_filters, help_requests::help_requests_filters, rejections::handle_rejection,
};

#[derive(Serialize, Deserialize, Clone)]
enum UserType {
    Volunteer(Vec<String>),
    Senior(Option<String>),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    username: String,
    name: String,
    address: String,
    location: (f64, f64),
    user_type: UserType,
    salt: [u8; 32],
    password_hash: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize)]
enum HelpRequestState {
    Pending,
    AcceptedBy(String),
    MarkedCompletedBy(String),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HelpRequest {
    picture: String,
    notes: String,
    creation_time: i64,
    state: HelpRequestState,
    username: String,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let db = Db::open("users");
    let help_requests_db = Db::open("help-requests");

    let accounts = accounts_filters(&db);
    let help_requests = help_requests_filters(&db, &help_requests_db);

    let get = warp::get().and(warp::fs::dir("../frontend/build"));
    let post = warp::post().and(accounts);

    let routes = get.or(post).or(help_requests).recover(handle_rejection);

    info!("Serving");

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}
