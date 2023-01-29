mod accounts;
mod authorization;
mod db;
mod rejections;

use db::Db;
use log::info;
use serde::{Deserialize, Serialize};
use warp::Filter;

use crate::{accounts::accounts_filters, rejections::handle_rejection};

#[derive(Serialize, Deserialize, Clone, Copy)]
enum UserType {
    Volunteer,
    Senior,
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

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let db = Db::open("users");

    let accounts = accounts_filters(&db);

    let get = warp::get().and(warp::fs::dir("../frontend/build"));
    let post = warp::post().and(accounts);

    let routes = get.or(post).recover(handle_rejection);

    info!("Serving");

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}
