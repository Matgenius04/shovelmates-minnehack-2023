mod authorization;
mod db;

use authorization::{create_token, hash_password};
use db::UserDb;
use log::{error, info, trace, warn};
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use warp::{hyper::Response, Filter};

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

#[derive(Deserialize)]
struct CreateAccountInfo {
    username: String,
    name: String,
    address: String,
    location: (f64, f64),
    user_type: UserType,
    password: Secret<String>,
}

#[derive(Clone, Deserialize)]
struct LoginInfo {
    username: String,
    password: Secret<String>,
}

fn create_account(
    db: &UserDb,
    create_account_info: CreateAccountInfo,
) -> Result<Response<String>, anyhow::Error> {
    trace!(
        "Attempting to create an account for {}",
        &create_account_info.username
    );

    if db.contains(&create_account_info.username)? {
        trace!("Attempted to create an account that already exists");

        return Ok(Response::builder()
            .status(409)
            .body("The username already exists".to_owned())?);
    }

    let salt = rand::random::<[u8; 32]>();

    let password_hash = hash_password(&create_account_info.password, salt);

    let user = User {
        username: create_account_info.username.to_owned(),
        name: create_account_info.name,
        address: create_account_info.address,
        location: create_account_info.location,
        user_type: create_account_info.user_type,
        salt,
        password_hash,
    };

    db.add(&user)?;

    info!(
        "Created a new account for {}",
        &create_account_info.username
    );

    Ok(Response::builder()
        .status(200)
        .body(create_token(&create_account_info.username)?)?)
}

fn login(db: &UserDb, login_info: LoginInfo) -> Result<Response<String>, anyhow::Error> {
    trace!("Login attempt for {}", &login_info.username);

    let user = match db.get(&login_info.username)? {
        Some(user) => user,
        None => {
            trace!(
                "Login attempt failed because `{}` doesn't exist",
                &login_info.username
            );

            return Ok(Response::builder()
                .status(409)
                .body("The username doesn't exist".to_string())?);
        }
    };

    if user.password_hash != hash_password(&login_info.password, user.salt) {
        warn!(
            "Login attempt for `{}` failed because of incorrect password",
            &login_info.username
        );

        return Ok(Response::builder()
            .status(403)
            .body("The password is incorrect".to_owned())?);
    }

    info!("{} logged in", &login_info.username);

    Ok(Response::builder()
        .status(200)
        .body(create_token(&login_info.username)?)?)
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let db = UserDb::open("users");

    let create_account_db = db.to_owned();
    let create_account = warp::path!("api" / "create-account")
        .and(warp::body::json::<CreateAccountInfo>())
        .map(move |create_account_info: CreateAccountInfo| {
            match create_account(&create_account_db, create_account_info) {
                Ok(reply) => Ok(reply),
                Err(e) => {
                    error!("There was an error creating an account: {e:?}");
                    Response::builder().status(500).body(e.to_string())
                }
            }
        });

    let login_db = db.to_owned();
    let login = warp::path!("api" / "login")
        .and(warp::body::json::<LoginInfo>())
        .map(
            move |login_info: LoginInfo| match login(&login_db, login_info) {
                Ok(reply) => Ok(reply),
                Err(e) => {
                    error!("There was an error logging someone in: {e:?}");
                    Response::builder().status(500).body(e.to_string())
                }
            },
        );

    let get = warp::get().and(warp::fs::dir("../frontend/build"));
    let post = warp::post().and(create_account.or(login));

    let routes = get.or(post);

    info!("Serving");

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}
