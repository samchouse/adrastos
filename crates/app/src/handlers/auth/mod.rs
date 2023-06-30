use actix_session::Session;
use adrastos_core::{
    auth::{self, TokenType},
    config,
    entities::{UpdateUser, User},
    error::Error,
    id::Id,
    util,
};
use tokio::sync::Mutex;

use actix_web::{cookie::Cookie, get, post, web, HttpRequest, HttpResponse, Responder};
use chrono::{Duration, Utc};
use deadpool_redis::redis::{self, AsyncCommands};
use futures_util::StreamExt;
use lettre::{
    message::header::ContentType, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;

use crate::{middleware::user::RequiredUser, openapi, session::SessionKey};

pub mod mfa;
pub mod oauth2;
pub mod token;

#[derive(Deserialize)]
pub struct VerifyParams {
    token: String,
}

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
    config: web::Data<Mutex<config::Config>>,
    db_pool: web::Data<deadpool_postgres::Pool>,
    redis_pool: web::Data<deadpool_redis::Pool>,
    mailer: web::Data<Option<AsyncSmtpTransport<Tokio1Executor>>>,
) -> actix_web::Result<impl Responder, Error> {
    if !mailchecker::is_valid(body.email.as_str()) {
        return Err(Error::BadRequest("Invalid email".into()));
    }

    if User::find()
        .by_email(body.email.clone())
        .one(&db_pool)
        .await
        .is_ok()
    {
        return Err(Error::BadRequest("Email already in use".into()));
    }

    if User::find()
        .by_username(body.username.clone())
        .one(&db_pool)
        .await
        .is_ok()
    {
        return Err(Error::BadRequest("Username already in use".into()));
    }

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

    if let Some(mailer) = mailer.get_ref() {
        let verification_token = Id::new().to_string();

        let mut conn = redis_pool.get().await.unwrap();
        redis::cmd("SETEX")
            .arg(format!("verification:{}", verification_token))
            .arg(Duration::hours(1).num_seconds())
            .arg(user.id.clone())
            .query_async(&mut conn)
            .await
            .map_err(|_| {
                Error::InternalServerError(
                    "An error ocurred while saving verification token to Redis".into(),
                )
            })?;

        let mut conn = redis::Client::open(config.lock().await.redis_url.clone())
            .map_err(|_| Error::InternalServerError("Unable to connect to Redis".into()))?
            .get_async_connection()
            .await
            .map_err(|_| Error::InternalServerError("Unable to connect to Redis".into()))?;
        conn.publish::<_, _, ()>("emails", verification_token)
            .await
            .unwrap();

        let mut pubsub = conn.into_pubsub();
        pubsub.subscribe("html").await.map_err(|_| {
            // TODO(@Xenfo): set a timeout
            Error::InternalServerError("An error occurred while subscribing to Redis".into())
        })?;

        let mut stream = pubsub.on_message();
        let Some(msg) = stream.next().await else {
            return Err(Error::InternalServerError(
                "An error occurred while receiving Redis message".into(),
            ));
        };

        drop(stream);
        pubsub.unsubscribe("html").await.map_err(|_| {
            Error::InternalServerError("An error occurred while unsubscribing from Redis".into())
        })?;

        let html = msg.get_payload::<String>().unwrap();
        let message = Message::builder()
            .from("Adrastos <no-reply@adrastos.xenfo.dev>".parse().unwrap())
            .to(format!("<{}>", body.email).parse().unwrap())
            .subject("Verify Your Email")
            .header(ContentType::TEXT_HTML)
            .body(html)
            .unwrap();

        mailer.send(message).await.map_err(|_| {
            Error::InternalServerError(
                "An error occurred while sending the verification email".into(),
            )
        })?;
    }

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
    config: web::Data<Mutex<config::Config>>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let user = User::find()
        .by_email(body.email.clone())
        .one(&db_pool)
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

    let auth = auth::authenticate(&db_pool, &config.lock().await.clone(), &user).await?;
    Ok(HttpResponse::Ok()
        .cookie(auth.cookie.clone())
        .cookie(
            Cookie::build("isLoggedIn", true.to_string())
                .secure(true)
                .http_only(true)
                .path("/")
                .expires(auth.cookie.expires().unwrap())
                .finish(),
        )
        .json(json!({
            "success": true,
            "user": user
        })))
}

#[get("/logout")]
pub async fn logout(
    req: HttpRequest,
    user: RequiredUser,
    config: web::Data<Mutex<config::Config>>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let mut cookies = util::get_auth_cookies(&req)?;
    let refresh_token = auth::TokenType::verify(
        &config.lock().await.clone(),
        cookies.refresh_token.value().into(),
    )?;
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

    cookies.is_logged_in.make_removal();
    cookies.refresh_token.make_removal();
    Ok(HttpResponse::Ok()
        .cookie(cookies.is_logged_in)
        .cookie(cookies.refresh_token)
        .json(json!({
            "success": true
        })))
}

#[get("/verify")]
pub async fn verify(
    user: RequiredUser,
    params: web::Query<VerifyParams>,
    db_pool: web::Data<deadpool_postgres::Pool>,
    redis_pool: web::Data<deadpool_redis::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    if user.verified {
        return Err(Error::BadRequest("User is already verified".into()));
    }

    let mut conn = redis_pool.get().await.unwrap();
    let user_id: String = redis::cmd("GET")
        .arg(format!("verification:{}", params.token))
        .query_async(&mut conn)
        .await
        .map_err(|_| {
            Error::InternalServerError(
                "An error ocurred while getting verification token from Redis".into(),
            )
        })?;

    if user_id != user.id {
        return Err(Error::BadRequest("Invalid verification token".into()));
    }

    user.update(
        &db_pool,
        UpdateUser {
            verified: Some(true),
            ..Default::default()
        },
    )
    .await
    .map_err(|_| Error::InternalServerError("Unable to update user".to_string()))?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Email was successfully verified"
    })))
}
