use actix_web::{
    cookie::{time::OffsetDateTime, Cookie, Expiration},
    get, post, web, HttpRequest, HttpResponse, Responder,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;

use crate::{
    auth::{self, TokenType},
    config,
    entities::{Mutate, RefreshTokenTree, User},
    handlers::Error,
    id::Id,
    openapi,
};

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
        (status = 400, description = "Validation failed"),
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
    body: web::Json<LoginBody>,
    config: web::Data<config::Config>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let user = User::select()
        .by_email(&body.email)
        .finish(&db_pool)
        .await?;

    let is_valid = auth::verify_password(body.password.as_str(), &user.password).map_err(|_| {
        Error::BadRequest {
            message: "Unable to parse password hash".into(),
        }
    })?;
    if !is_valid {
        return Err(Error::BadRequest {
            message: "No user was found with this email/password combo".into(),
        });
    }

    let access_token =
        TokenType::Access
            .sign(&config, &user)
            .map_err(|_| Error::InternalServerError {
                error: "An error occurred while signing the access token".into(),
            })?;
    let refresh_token =
        TokenType::Refresh
            .sign(&config, &user)
            .map_err(|_| Error::InternalServerError {
                error: "An error occurred while signing the refresh token".into(),
            })?;

    let refresh_token_tree = RefreshTokenTree {
        id: Id::new().to_string(),
        user_id: user.id.clone(),
        inactive_at: Utc::now() + chrono::Duration::days(15),
        expires_at: Utc::now() + chrono::Duration::days(90),
        tokens: vec![refresh_token.clone().claims.jti],
        created_at: Utc::now(),
        updated_at: None,
    };

    refresh_token_tree.create(&db_pool).await?;

    let cookie_expiration = OffsetDateTime::from_unix_timestamp(
        refresh_token.expires_at.timestamp(),
    )
    .map_err(|_| Error::InternalServerError {
        error: "An error occurred while parsing the cookie expiration".into(),
    })?;

    Ok(HttpResponse::Ok()
        .cookie(
            Cookie::build("refreshToken", refresh_token.token)
                .secure(true)
                .http_only(true)
                .expires(Expiration::from(cookie_expiration))
                .finish(),
        )
        .json(json!({
            "success": true,
            "user": user,
            "accessToken": access_token.clone().token,
        })))
}

#[get("/logout")]
pub async fn logout(req: HttpRequest) -> actix_web::Result<impl Responder, Error> {
    let cookies = req.cookies().map_err(|_| Error::InternalServerError {
        error: "An error occurred while fetching the cookies".into(),
    })?;

    let cookie = cookies
        .iter()
        .find(|cookie| cookie.name() == "refreshToken")
        .ok_or_else(|| Error::BadRequest {
            message: "No refresh token was found".into(),
        })?;

    let mut cookie = cookie.clone();
    cookie.make_removal();

    Ok(HttpResponse::Ok().cookie(cookie).json(json!({
        "success": true
    })))
}
