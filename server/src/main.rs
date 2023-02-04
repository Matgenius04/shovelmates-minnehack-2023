mod accounts;
mod authorization;
mod db;
mod errors;
mod help_requests;
mod volunteering;

use std::convert::Infallible;

use db::{Archived, Db};
use geo::algorithm::geodesic_distance::GeodesicDistance;
use log::info;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use warp::{
    filters::any,
    hyper::{body::Bytes, Body, Response},
    Filter,
};

use crate::{
    accounts::accounts_filters, errors::Error, help_requests::help_requests_filters,
    volunteering::volunteering_filters,
};

#[derive(Serialize, Deserialize, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
pub enum UserType {
    Volunteer(Vec<String>),
    Senior(Option<String>),
}

#[derive(Clone, Copy, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(as = "Self")]
pub struct Location(f64, f64);

impl From<Location> for geo::Point {
    fn from(value: Location) -> Self {
        geo::Point::new(value.0, value.1)
    }
}

impl From<(f64, f64)> for Location {
    fn from((a, b): (f64, f64)) -> Self {
        Location(a, b)
    }
}

impl From<Location> for (f64, f64) {
    fn from(Location(a, b): Location) -> Self {
        (a, b)
    }
}

#[derive(Clone, Archive, RkyvSerialize, RkyvDeserialize)]
#[repr(C)]
pub struct User {
    username: String,
    name: String,
    address: String,
    location: Location,
    user_type: UserType,
    salt: [u8; 32],
    password_hash: Vec<u8>,
}

pub type UserDB = Db<250, User>;

#[derive(Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
pub enum HelpRequestState {
    Pending,
    AcceptedBy(String),
    MarkedCompletedBy(String),
}

impl ArchivedHelpRequestState {
    fn to_json(&self) -> serde_json::Value {
        match self {
            ArchivedHelpRequestState::Pending => json!("Pending"),
            ArchivedHelpRequestState::AcceptedBy(user) => json!({ "AcceptedBy": &**user }),
            ArchivedHelpRequestState::MarkedCompletedBy(user) => {
                json!({ "MarkedCompletedBy": &**user })
            }
        }
    }
}

#[derive(Clone, Archive, RkyvSerialize, RkyvDeserialize)]
#[repr(C)]
pub struct HelpRequest {
    picture: String,
    notes: String,
    creation_time: i64,
    state: HelpRequestState,
    username: String,
}

pub type HelpRequestDB = Db<150, HelpRequest>;

pub fn distance_meters(coord1: Location, coord2: Location) -> f64 {
    geo::Point::from(coord1).geodesic_distance(&coord2.into())
}

impl HelpRequest {
    pub fn get_user(&self, user_db: &UserDB) -> Result<Archived<User>, anyhow::Error> {
        user_db.get(&self.username)?.ok_or_else(|| {
            anyhow::Error::msg("The username in the help request doesn't exist in the database")
        })
    }

    pub fn distance_meters(
        &self,
        coordinates: Location,
        user_db: &UserDB,
    ) -> Result<f64, anyhow::Error> {
        let senior = self.get_user(user_db)?;
        let senior_coord = senior.location;

        Ok(distance_meters(coordinates, senior_coord))
    }
}

impl ArchivedHelpRequest {
    pub fn get_user(&self, user_db: &UserDB) -> Result<Archived<User>, anyhow::Error> {
        user_db.get(&self.username)?.ok_or_else(|| {
            anyhow::Error::msg("The username in the help request doesn't exist in the database")
        })
    }

    pub fn distance_meters(
        &self,
        coordinates: Location,
        user_db: &UserDB,
    ) -> Result<f64, anyhow::Error> {
        let senior = self.get_user(user_db)?;
        let senior_coord = senior.location;

        Ok(distance_meters(coordinates, senior_coord))
    }
}

trait InfallibleDeserialize<T>: RkyvDeserialize<T, rkyv::Infallible> {
    fn deserialize(&self) -> T {
        RkyvDeserialize::deserialize(self, &mut rkyv::Infallible).unwrap()
    }
}

impl<V, T: RkyvDeserialize<V, rkyv::Infallible>> InfallibleDeserialize<V> for T {}

pub fn extract_json<T: DeserializeOwned>(bytes: &Bytes) -> Result<T, Error> {
    serde_json::from_slice(bytes.as_ref()).map_err(|e| Error::JSON(e))
}

pub fn clone<V: Clone + Send>(v: V) -> impl Filter<Extract = (V,), Error = Infallible> + Clone {
    any::any().map(move || v.to_owned())
}

pub fn clone_dbs(
    user_db: &UserDB,
    requests_db: &HelpRequestDB,
) -> impl Filter<Extract = (UserDB, HelpRequestDB), Error = Infallible> + Clone {
    clone(user_db.to_owned()).and(clone(requests_db.to_owned()))
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let db = sled::open("db").expect("the DB to open properly");
    let users_db: UserDB = Db::open(&db, "users");
    let help_requests_db: HelpRequestDB = Db::open(&db, "help-requests");

    let accounts = accounts_filters(&users_db);
    let help_requests = help_requests_filters(&users_db, &help_requests_db);
    let volunteering = volunteering_filters(&users_db, &help_requests_db);

    let get = warp::get().and(warp::fs::dir("../frontend/build"));
    let post = warp::post()
        .and(accounts.or(help_requests).unify().or(volunteering).unify())
        .map(|v: Result<Response<Body>, Error>| match v {
            Ok(v) => v,
            Err(e) => e.into_response(),
        });

    let routes = get.or(post).with(
        warp::cors()
            .allow_any_origin()
            .allow_methods(["GET", "POST"])
            .allow_headers(["Content-Type"]),
    );

    info!("Serving");

    warp::serve(routes.to_owned())
        .run(([0, 0, 0, 0], 8080))
        .await;
    // let https = warp::serve(routes)
    //     .tls()
    //     .cert(include_bytes!("../../self-signed-bs/certificate.pem"))
    //     .key(include_bytes!("../../self-signed-bs/key.pem"))
    //     .run(([0, 0, 0, 0], 8079));
}
