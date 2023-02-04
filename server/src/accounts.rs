use log::{debug, info};
use secrecy::Secret;
use serde::Deserialize;
use serde_json::json;
use warp::{hyper::Response, reject, Filter, Rejection};

use crate::{
    authorization::{authorize, create_token, hash_password},
    rejections::CustomRejection,
    InfallibleDeserialize, Location, User, UserDB, UserType,
};

#[derive(Deserialize, Clone, Copy)]
enum UserTypeChoice {
    Volunteer,
    Senior,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateAccountInfo {
    username: String,
    name: String,
    address: String,
    location: (f64, f64),
    user_type: UserTypeChoice,
    password: Secret<String>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginInfo {
    username: String,
    password: Secret<String>,
}

pub fn accounts_filters(
    db: &UserDB,
) -> impl Filter<Extract = (Response<String>,), Error = Rejection> + Clone {
    let create_account_db = db.to_owned();
    let create_account = warp::path!("api" / "create-account")
        .and(warp::body::json::<CreateAccountInfo>())
        .and_then(move |create_account_info: CreateAccountInfo| {
            let db = create_account_db.to_owned();
            async move { create_account(&db, create_account_info).map_err(reject::custom) }
        });

    let login_db = db.to_owned();
    let login = warp::path!("api" / "login")
        .and(warp::body::json::<LoginInfo>())
        .and_then(move |login_info: LoginInfo| {
            let db = login_db.to_owned();
            async move { login(&db, login_info).map_err(reject::custom) }
        });

    let account_info_db = db.to_owned();
    let account_info =
        warp::path!("api" / "user-data")
            .and(authorize())
            .and_then(move |username| {
                let db = account_info_db.to_owned();
                async move { get_account_info(username, &db).map_err(reject::custom) }
            });

    warp::post().and(create_account.or(login).unify().or(account_info).unify())
}

fn create_account(
    db: &UserDB,
    create_account_info: CreateAccountInfo,
) -> Result<Response<String>, CustomRejection> {
    debug!(
        "Attempting to create an account for {}",
        &create_account_info.username
    );

    if db.contains(&create_account_info.username)? {
        return Err(CustomRejection::UsernameAlreadyExists(
            create_account_info.username,
        ));
    }

    let salt = rand::random::<[u8; 32]>();

    let password_hash = hash_password(&create_account_info.password, salt);

    let user = User {
        username: create_account_info.username.to_owned(),
        name: create_account_info.name,
        address: create_account_info.address,
        location: create_account_info.location.into(),
        user_type: match create_account_info.user_type {
            UserTypeChoice::Volunteer => UserType::Volunteer(Vec::new()),
            UserTypeChoice::Senior => UserType::Senior(None),
        },
        salt,
        password_hash,
    };

    db.add(&user.name, &user)?;

    info!(
        "Created a new account for {}",
        &create_account_info.username
    );

    Ok(Response::builder()
        .status(200)
        .body(create_token(&create_account_info.username)?)?)
}

fn login(db: &UserDB, login_info: LoginInfo) -> Result<Response<String>, CustomRejection> {
    debug!("Login attempt for {}", &login_info.username);

    let user = match db.get(&login_info.username)? {
        Some(user) => user,
        None => {
            return Err(CustomRejection::UsernameDoesntExist(login_info.username));
        }
    };

    if user.password_hash != hash_password(&login_info.password, user.salt) {
        return Err(CustomRejection::IncorrectPassword(login_info.username));
    }

    info!("{} logged in", &login_info.username);

    Ok(Response::builder()
        .status(200)
        .body(create_token(&login_info.username)?)?)
}

fn get_account_info(username: String, db: &UserDB) -> Result<Response<String>, CustomRejection> {
    let user = match db.get(&username)? {
        Some(v) => v,
        None => {
            return Err(CustomRejection::Anyhow(anyhow::Error::msg(
                "The username doesn't exist in the database",
            )))
        }
    };

    info!("{username} requested their user data");

    Ok(Response::builder()
        .status(200)
        .body(serde_json::to_string(&json!({
            "username": &*user.username,
            "name": &*user.name,
            "address": &*user.address,
            "location": <(f64, f64) as From<Location>>::from(user.location),
            "user_type": InfallibleDeserialize::<UserType>::deserialize(&user.user_type),
        }))?)?)
}
