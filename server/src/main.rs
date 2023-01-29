mod accounts;
mod authorization;
mod db;
mod help_requests;
mod rejections;
mod volunteering;

use db::Db;
use geo::algorithm::geodesic_distance::GeodesicDistance;
use log::info;
use serde::{Deserialize, Serialize};
use tokio::join;
use warp::Filter;

use crate::{
    accounts::accounts_filters, help_requests::help_requests_filters, rejections::handle_rejection,
    volunteering::volunteering_filters,
};

#[derive(Serialize, Deserialize, Clone)]
enum UserType {
    Volunteer(Vec<String>),
    Senior(Option<String>),
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct HelpRequest {
    picture: String,
    notes: String,
    creation_time: i64,
    state: HelpRequestState,
    username: String,
}

pub fn distance_meters(coord1: (f64, f64), coord2: (f64, f64)) -> f64 {
    geo::Point(geo::Coord {
        x: coord1.0,
        y: coord1.1,
    })
    .geodesic_distance(&geo::Point(geo::Coord {
        x: coord2.0,
        y: coord2.1,
    }))
}

impl HelpRequest {
    pub fn get_user(&self, user_db: &Db<User>) -> Result<User, anyhow::Error> {
        user_db.get(&self.username)?.ok_or_else(|| {
            anyhow::Error::msg("The username in the help request doesn't exist in the database")
        })
    }

    pub fn distance_meters(
        &self,
        coordinates: (f64, f64),
        user_db: &Db<User>,
    ) -> Result<f64, anyhow::Error> {
        let senior = self.get_user(user_db)?;
        let senior_coord = senior.location;

        Ok(geo::Point(geo::Coord {
            x: coordinates.0,
            y: coordinates.1,
        })
        .geodesic_distance(&geo::Point(geo::Coord {
            x: senior_coord.0,
            y: senior_coord.1,
        })))
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let db = Db::open("users");
    let help_requests_db = Db::open("help-requests");

    let accounts = accounts_filters(&db);
    let help_requests = help_requests_filters(&db, &help_requests_db);
    let volunteering = volunteering_filters(&db, &help_requests_db);

    let get = warp::get().and(warp::fs::dir("../frontend/build"));
    let post = warp::post().and(accounts.or(help_requests).or(volunteering));

    let routes = get.or(post).recover(handle_rejection);

    info!("Serving");

    let http = warp::serve(routes.to_owned())
        .run(([0, 0, 0, 0], 8080))
        .await;
    // let https = warp::serve(routes)
    //     .tls()
    //     .cert(include_bytes!("../../self-signed-bs/certificate.pem"))
    //     .key(include_bytes!("../../self-signed-bs/key.pem"))
    //     .run(([0, 0, 0, 0], 8079));
}
