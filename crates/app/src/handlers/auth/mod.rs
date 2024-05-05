use adrastos_core::{
    auth::{self, TokenType},
    entities::{self, UserType},
    error::Error,
    id::Id,
    util,
};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use chrono::{Duration, Utc};
use fred::{
    interfaces::{EventInterface, KeysInterface, PubsubInterface},
    types::Expiration,
};
use lettre::{message::header::ContentType, AsyncTransport, Message};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::time::{self, timeout};
use tower_sessions::Session;
use tracing::{error, warn};

use crate::{
    middleware::extractors::{AnyUser, Config, Database, Mailer, ProjectDatabase, User},
    session::SessionKey,
    state::AppState,
};

pub mod mfa;
pub mod oauth2;
pub mod passkeys;
pub mod token;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterBody {
    first_name: String,
    last_name: String,
    email: String,
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginBody {
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct VerifyParams {
    token: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", get(logout))
        .route("/verify", get(verify))
        .route("/resend-verification", post(resend_verification))
        .nest("/mfa", mfa::routes())
        .nest("/token", token::routes())
        .nest("/passkeys", passkeys::routes())
        .nest("/oauth2", oauth2::routes())
}

pub async fn register(
    mailer: Option<Mailer>,
    Config(config): Config,
    Database(db): Database,
    State(AppState {
        redis_pool,
        subscriber,
        ..
    }): State<AppState>,
    Json(body): Json<RegisterBody>,
) -> Result<impl IntoResponse, Error> {
    if !mailchecker::is_valid(body.email.as_str()) {
        return Err(Error::BadRequest("Invalid email".into()));
    }

    if UserType::from(&db)
        .find()
        .by_email(body.email.clone())
        .one()
        .await
        .is_ok()
    {
        return Err(Error::BadRequest("Email already in use".into()));
    }

    if UserType::from(&db)
        .find()
        .by_username(body.username.clone())
        .one()
        .await
        .is_ok()
    {
        return Err(Error::BadRequest("Username already in use".into()));
    }

    let user = entities::AnyUser {
        id: Id::new().to_string(),
        first_name: body.first_name.clone(),
        last_name: body.last_name.clone(),
        username: body.username.clone(),
        email: body.email.clone(),
        password: body.password.clone(),
        created_at: Utc::now(),
        ..Default::default()
    };

    UserType::from(&db).create(user.clone()).await?;

    if let UserType::Normal(_) = UserType::from(&db) {
        if let Some(Mailer(mailer)) = mailer {
            let verification_token = Id::new().to_string();
            let channel = format!("html:{}", verification_token);

            redis_pool
                .set(
                    format!("verification:{}", verification_token),
                    user.id.clone(),
                    Some(Expiration::EX(Duration::hours(1).num_seconds())),
                    None,
                    false,
                )
                .await
                .map_err(|_| {
                    Error::InternalServerError(
                        "An error ocurred while saving verification token to Redis".into(),
                    )
                })?;

            let (c_channel, c_user) = (channel.clone(), user.clone());
            let mut message_rx = subscriber.message_rx();
            let task = tokio::spawn(async move {
                while let Ok(Ok(message)) =
                    timeout(time::Duration::from_secs(3), message_rx.recv()).await
                {
                    if message.channel != c_channel {
                        continue;
                    }

                    let message = Message::builder()
                        .from(
                            format!(
                                "{} <{}>",
                                config.smtp_sender_name.unwrap(),
                                config.smtp_sender_email.unwrap()
                            )
                            .parse()
                            .unwrap(),
                        )
                        .to(format!("<{}>", body.email).parse().unwrap())
                        .subject("Verify Your Email")
                        .header(ContentType::TEXT_HTML)
                        .body(message.value.as_str().unwrap().to_string())
                        .unwrap();

                    let _ = mailer.send(message).await;
                    return;
                }

                warn!(
                    user.id = c_user.id,
                    "Redis timed out or errored while waiting for verification email"
                );
            });

            subscriber.subscribe(channel.clone()).await.map_err(|_| {
                Error::InternalServerError("An error occurred while subscribing to Redis".into())
            })?;

            subscriber
                .publish("emails", verification_token)
                .await
                .map_err(|_| {
                    Error::InternalServerError("An error occurred while publishing to Redis".into())
                })?;

            let _ = task.await;
            subscriber.unsubscribe(channel).await.unwrap();
        }
    }

    Ok(Json(user).into_response())
}

pub async fn login(
    jar: CookieJar,
    session: Session,
    Config(config): Config,
    Database(db): Database,
    Json(body): Json<LoginBody>,
) -> Result<impl IntoResponse, Error> {
    let user = UserType::from(&db)
        .find()
        .by_email(body.email.clone())
        .one()
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
            .insert(&SessionKey::LoginUserId.to_string(), user.id)
            .await
            .map_err(|_| {
                Error::InternalServerError("An error occurred while setting the session".into())
            })?;
        session
            .insert(&SessionKey::MfaRetries.to_string(), 3)
            .await
            .map_err(|_| {
                Error::InternalServerError("An error occurred while setting the session".into())
            })?;

        return Ok(Json(json!({
            "success": true,
            "message": "MFA is required for this user, continue to MFA verification",
        }))
        .into_response());
    }

    let (_, jar) = auth::authenticate(&db, &config.clone(), &user, jar).await?;
    Ok((jar, Json(user).into_response()).into_response())
}

pub async fn logout(
    jar: CookieJar,
    Config(config): Config,
    AnyUser(user): AnyUser,
    Database(db): Database,
) -> Result<impl IntoResponse, Error> {
    let cookies = util::get_auth_cookies(&jar)?;
    let refresh_token =
        auth::TokenType::verify(&config.clone(), cookies.refresh_token.value().into())?;
    if refresh_token.token_type != TokenType::Refresh {
        return Err(Error::Unauthorized);
    }

    user.refresh_token_trees
        .clone()
        .ok_or_else(|| Error::Unauthorized)?
        .into_iter()
        .find(|tree| tree.tokens.contains(&refresh_token.claims.jti))
        .ok_or_else(|| Error::Unauthorized)?
        .delete(&db)
        .await?;

    let _ = jar.remove("isLoggedIn").remove("refreshToken");
    Ok(Json(Value::Null))
}

pub async fn verify(
    jar: CookieJar,
    Config(config): Config,
    ProjectDatabase(db): ProjectDatabase,
    Query(params): Query<VerifyParams>,
    State(AppState { redis_pool, .. }): State<AppState>,
) -> Result<impl IntoResponse, Error> {
    // TODO(@Xenfo): make this middleware
    let refresh_token = auth::TokenType::verify(
        &config.clone(),
        util::get_auth_cookies(&jar)?.refresh_token.value().into(),
    )?;
    if refresh_token.token_type != TokenType::Refresh {
        return Err(Error::Forbidden("Not a refresh token".into()));
    }

    let user = entities::User::find_by_id(&refresh_token.claims.sub)
        .one(&db)
        .await?;

    if user.verified {
        return Err(Error::BadRequest("User is already verified".into()));
    }

    let user_id: String = redis_pool
        .get(format!("verification:{}", params.token))
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
        &db,
        entities::UpdateUser {
            verified: Some(true),
            ..Default::default()
        },
    )
    .await
    .map_err(|_| Error::InternalServerError("Unable to update user".to_string()))?;

    Ok(Json(Value::Null))
}

pub async fn resend_verification(
    User(user): User,
    Config(config): Config,
    Mailer(mailer): Mailer,
    State(AppState {
        redis_pool,
        subscriber,
        ..
    }): State<AppState>,
) -> Result<impl IntoResponse, Error> {
    if user.verified {
        return Err(Error::BadRequest("User is already verified".into()));
    }

    let verification_token = Id::new().to_string();
    let channel = format!("html:{}", verification_token);

    redis_pool
        .set(
            format!("verification:{}", verification_token),
            user.id.clone(),
            Some(Expiration::EX(Duration::hours(1).num_seconds())),
            None,
            false,
        )
        .await
        .map_err(|_| {
            Error::InternalServerError(
                "An error ocurred while saving verification token to Redis".into(),
            )
        })?;

    let (c_channel, c_user) = (channel.clone(), user.clone());
    let mut message_rx = subscriber.message_rx();
    let task = tokio::spawn(async move {
        while let Ok(Ok(message)) = timeout(time::Duration::from_secs(3), message_rx.recv()).await {
            if message.channel != c_channel {
                continue;
            }

            let message = Message::builder()
                .from(
                    format!(
                        "{} <{}>",
                        config.smtp_sender_name.unwrap(),
                        config.smtp_sender_email.unwrap()
                    )
                    .parse()
                    .unwrap(),
                )
                .to(format!("<{}>", user.email).parse().unwrap())
                .subject("Verify Your Email")
                .header(ContentType::TEXT_HTML)
                .body(message.value.as_str().unwrap().to_string())
                .unwrap();

            mailer.send(message).await.map_err(|_| {
                Error::InternalServerError(
                    "An error occurred while sending the verification email".into(),
                )
            })?;

            return Ok(());
        }

        error!(
            user.id = c_user.id,
            "Redis timed out or errored while waiting for verification email"
        );
        Err(Error::InternalServerError(
            "Redis timed out or errored while waiting for verification email".into(),
        ))
    });

    subscriber.subscribe(channel.clone()).await.map_err(|_| {
        Error::InternalServerError("An error occurred while subscribing to Redis".into())
    })?;

    subscriber
        .publish("emails", verification_token)
        .await
        .map_err(|_| {
            Error::InternalServerError("An error occurred while publishing to Redis".into())
        })?;

    let _ = task.await;
    subscriber.unsubscribe(channel).await.unwrap();

    Ok(Json(Value::Null))
}
