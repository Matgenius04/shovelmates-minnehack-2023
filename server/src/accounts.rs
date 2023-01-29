use log::{info, trace};
use secrecy::Secret;
use serde::Deserialize;
use warp::{hyper::Response, reject, Filter, Rejection};

use crate::{
    authorization::{create_token, hash_password},
    db::UserDb,
    rejections::CustomRejection,
    User, UserType,
};

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

pub fn accounts_filters(
    db: &UserDb,
) -> impl Filter<Extract = (Response<String>,), Error = Rejection> + Clone {
    let create_account_db = db.to_owned();
    let create_account = warp::path!("api" / "create-account")
        .and(warp::body::json::<CreateAccountInfo>())
        .and_then(move |create_account_info: CreateAccountInfo| {
            let db = create_account_db.to_owned();
            async move { create_account(&db, create_account_info).map_err(|e| reject::custom(e)) }
        });

    let login_db = db.to_owned();
    let login = warp::path!("api" / "login")
        .and(warp::body::json::<LoginInfo>())
        .and_then(move |login_info: LoginInfo| {
            let db = login_db.to_owned();
            async move { login(&db, login_info).map_err(|e| reject::custom(e)) }
        });

    warp::post().and(create_account.or(login).unify())
}

fn create_account(
    db: &UserDb,
    create_account_info: CreateAccountInfo,
) -> Result<Response<String>, CustomRejection> {
    trace!(
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

fn login(db: &UserDb, login_info: LoginInfo) -> Result<Response<String>, CustomRejection> {
    trace!("Login attempt for {}", &login_info.username);

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
