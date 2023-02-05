use log::{debug, info};
use secrecy::Secret;
use serde::Deserialize;
use serde_json::json;
use warp::{body::bytes, hyper::Body, Filter, Rejection};

use crate::{
    authorization::{authorize, create_token, hash_password},
    db::Transactional,
    errors::Error,
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
) -> impl Filter<Extract = (Result<Body, Error>,), Error = Rejection> + Clone {
    let create_account_db = db.to_owned();
    let create_account = warp::path!("api" / "create-account")
        .and(warp::body::json::<CreateAccountInfo>())
        .map(move |create_account_info: CreateAccountInfo| {
            let db = create_account_db.to_owned();
            create_account(&db, create_account_info)
        });

    let login_db = db.to_owned();
    let login = warp::path!("api" / "login")
        .and(warp::body::json::<LoginInfo>())
        .map(move |login_info: LoginInfo| {
            let db = login_db.to_owned();
            login(&db, login_info)
        });

    let account_info_db = db.to_owned();
    let account_info = warp::path!("api" / "user-data")
        .and(bytes())
        .map(move |bytes| {
            let username = authorize(&bytes)?;
            let db = account_info_db.to_owned();
            get_account_info(username, &db)
        });

    warp::post().and(create_account.or(login).unify().or(account_info).unify())
}

fn create_account(db: &UserDB, create_account_info: CreateAccountInfo) -> Result<Body, Error> {
    debug!(
        "Attempting to create an account for {}",
        &create_account_info.username
    );

    db.transaction(move |db| {
        if db.get(&create_account_info.username)?.is_some() {
            return Err(
                Error::UsernameAlreadyExists(create_account_info.username.to_owned()).into(),
            );
        }

        let salt = rand::random::<[u8; 32]>();

        let password_hash = hash_password(&create_account_info.password, salt);

        let user = User {
            username: create_account_info.username.to_owned(),
            name: create_account_info.name.to_owned(),
            address: create_account_info.address.to_owned(),
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

        Ok(Body::from(create_token(&create_account_info.username)?))
    })
    .map_err(|e| e.into())
}

fn login(db: &UserDB, login_info: LoginInfo) -> Result<Body, Error> {
    debug!("Login attempt for {}", &login_info.username);

    let user = match db.get(&login_info.username)? {
        Some(user) => user,
        None => {
            return Err(Error::UsernameDoesntExist(login_info.username));
        }
    };

    if user.password_hash != hash_password(&login_info.password, user.salt) {
        return Err(Error::IncorrectPassword(login_info.username));
    }

    info!("{} logged in", &login_info.username);

    Ok(Body::from(create_token(&login_info.username)?))
}

fn get_account_info(username: String, db: &UserDB) -> Result<Body, Error> {
    let user = match db.get(&username)? {
        Some(v) => v,
        None => {
            return Err(Error::Anyhow(anyhow::Error::msg(
                "The username doesn't exist in the database",
            )))
        }
    };

    info!("{username} requested their user data");

    Ok(Body::from(serde_json::to_string(&json!({
        "username": &*user.username,
        "name": &*user.name,
        "address": &*user.address,
        "location": <(f64, f64) as From<Location>>::from(user.location),
        "user_type": InfallibleDeserialize::<UserType>::deserialize(&user.user_type),
    }))?))
}
