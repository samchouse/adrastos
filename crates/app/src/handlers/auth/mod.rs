use actix_session::Session;
use adrastos_core::{
    auth::{self, TokenType},
    config,
    entities::{Mutate, User},
    error::Error,
    id::Id,
    util,
};

use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;

use crate::{middleware::user::RequiredUser, openapi, session::SessionKey};

pub mod mfa;
pub mod oauth2;
pub mod token;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignupBody {
    #[schema(max_length = 50)]
    first_name: String,
    #[schema(max_length = 50)]
    last_name: String,
    #[schema(schema_with = openapi::email)]
    email: String,
    #[schema(min_length = 5, max_length = 64)]
    username: String,
    #[schema(min_length = 8, max_length = 64)]
    password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginBody {
    email: String,
    password: String,
}

#[utoipa::path(
    post,
    path = "/auth/signup",
    request_body = SignupBody,
    responses(
        (status = 200, description = "User created successfully", body = User),
        (status = 400, description = "Validation failed", body = Error),
    )
)]
#[post("/signup")]
pub async fn signup(
    body: web::Json<SignupBody>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let user = User {
        id: Id::new().to_string(),
        first_name: body.first_name.clone(),
        last_name: body.last_name.clone(),
        username: body.username.clone(),
        email: body.email.clone(),
        password: body.password.clone(),
        verified: false,
        banned: false,
        mfa_secret: None,
        mfa_backup_codes: None,
        created_at: Utc::now(),
        updated_at: None,

        connections: None,
        refresh_token_trees: None,
    };

    user.create(&db_pool).await?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "user": user
    })))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginBody,
    responses(
        (status = 200, description = "User created successfully", body = User),
        (status = 400, description = "Validation failed"),
    )
)]
#[post("/login")]
pub async fn login(
    session: Session,
    body: web::Json<LoginBody>,
    config: web::Data<config::Config>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let user = User::select()
        .by_email(&body.email)
        .finish(&db_pool)
        .await?;

    let is_valid = auth::verify_password(body.password.as_str(), &user.password)
        .map_err(|_| Error::BadRequest("Unable to parse password hash".into()))?;
    if !is_valid {
        return Err(Error::BadRequest(
            "No user was found with this email/password combo".into(),
        ));
    }

    if user.mfa_secret.is_some() {
        session
            .insert(SessionKey::LoginUserId.to_string(), user.id)
            .map_err(|_| {
                Error::InternalServerError("An error occurred while setting the session".into())
            })?;
        session
            .insert(SessionKey::MfaRetries.to_string(), 3)
            .map_err(|_| {
                Error::InternalServerError("An error occurred while setting the session".into())
            })?;

        return Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "message": "MFA is required for this user, continue to MFA verification",
        })));
    }

    let auth = auth::authenticate(&db_pool, &config, &user).await?;
    Ok(HttpResponse::Ok().cookie(auth.cookie).json(json!({
        "success": true,
        "user": user,
        "accessToken": auth.token.clone().token,
    })))
}

#[get("/logout")]
pub async fn logout(
    req: HttpRequest,
    user: RequiredUser,
    config: web::Data<config::Config>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let mut cookie = util::get_refresh_token_cookie(&req)?;
    let refresh_token = auth::TokenType::verify(&config, cookie.value().into())?;
    if refresh_token.token_type != TokenType::Refresh {
        return Err(Error::Unauthorized);
    }

    user.refresh_token_trees
        .clone()
        .ok_or_else(|| Error::Unauthorized)?
        .into_iter()
        .find(|tree| tree.tokens.contains(&refresh_token.claims.jti))
        .ok_or_else(|| Error::Unauthorized)?
        .delete(&db_pool)
        .await?;

    cookie.make_removal();
    Ok(HttpResponse::Ok().cookie(cookie).json(json!({
        "success": true
    })))
}
