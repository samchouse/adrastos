use tokio::sync::Mutex;

use adrastos_core::{
    auth::{
        self,
        oauth2::{providers::OAuth2Provider, OAuth2, OAuth2LoginInfo},
    },
    config,
    entities::{Connection, User},
    error::Error,
    id::Id,
};

use actix_session::Session;
use actix_web::{cookie::Cookie, get, http::header, web, HttpResponse, Responder};
use chrono::Utc;
use serde::Deserialize;
use tracing::error;

use crate::{middleware::user, session::SessionKey};

#[derive(Deserialize)]
pub struct LoginParams {
    provider: String,
    to: Option<String>,
}

#[derive(Deserialize)]
pub struct CallbackParams {
    provider: String,
    state: String,
    code: String,
}

#[get("/login")]
pub async fn login(
    user: user::User,
    session: Session,
    oauth2: web::Data<OAuth2>,
    params: web::Query<LoginParams>,
    redis_pool: web::Data<deadpool_redis::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let provider = OAuth2Provider::try_from(params.provider.as_str())
        .map_err(|_| Error::BadRequest("An invalid provider was provided".into()))?;

    let (auth_url, csrf_token) = oauth2
        .initialize_login(provider, &redis_pool)
        .await
        .map_err(|_| Error::InternalServerError("Unable to initialize the OAuth login".into()))?;

    session
        .insert(
            SessionKey::CsrfToken.to_string(),
            csrf_token.secret().to_string(),
        )
        .map_err(|_| {
            Error::InternalServerError("Unable to insert CSRF token into session".to_string())
        })?;

    if let Some(user) = user.clone() {
        session
            .insert(SessionKey::UserId.to_string(), user.id)
            .map_err(|_| {
                Error::InternalServerError("Unable to insert user ID into session".to_string())
            })?;
    }

    if let Some(to) = params.to.clone() {
        session
            .insert(SessionKey::Redirect.to_string(), to)
            .map_err(|_| {
                Error::InternalServerError("Unable to insert redirect URL into session".to_string())
            })?;
    }

    Ok(HttpResponse::Found()
        .append_header((header::LOCATION, auth_url.to_string()))
        .finish())
}

#[get("/callback")]
pub async fn callback(
    config: web::Data<Mutex<config::Config>>,
    oauth2: web::Data<OAuth2>,
    params: web::Query<CallbackParams>,
    db_pool: web::Data<deadpool_postgres::Pool>,
    redis_pool: web::Data<deadpool_redis::Pool>,
    session: Session,
) -> actix_web::Result<impl Responder, Error> {
    let client_url = config.lock().await.client_url.clone();

    let provider = OAuth2Provider::try_from(params.provider.as_str())
        .map_err(|_| Error::BadRequest("An invalid provider was provided".into()))?;

    let Ok(Some(session_csrf_token)) = session.get::<String>(&SessionKey::CsrfToken.to_string())
    else {
        return Err(Error::BadRequest(
            "The request is missing a session CSRF Token".into(),
        ));
    };

    let token = oauth2
        .confirm_login(
            provider.clone(),
            &config,
            &redis_pool,
            OAuth2LoginInfo {
                session_csrf_token,
                params_csrf_token: params.state.to_string(),
                auth_code: params.code.to_string(),
            },
        )
        .await
        .map_err(Error::InternalServerError)?;

    let oauth2_user = provider.fetch_user(&oauth2, &token).await.map_err(|e| {
        error!("Unable to fetch the user from the OAuth provider: {}", e);
        Error::InternalServerError("Unable to fetch the user from the OAuth provider".into())
    })?;

    let connection = Connection::find()
        .by_provider(provider.to_string())
        .by_provider_id(oauth2_user.id.clone())
        .one(&db_pool)
        .await;

    let user = match connection {
        Ok(conn) => Ok(User::find_by_id(&conn.user_id).one(&db_pool).await?),
        Err(..) => {
            if let Ok(Some(user_id)) = session.get::<String>(&SessionKey::UserId.to_string()) {
                let conn = Connection {
                    id: Id::new().to_string(),
                    provider: provider.to_string(),
                    provider_id: oauth2_user.id.clone(),
                    user_id,
                    created_at: Utc::now(),
                    updated_at: None,
                };

                conn.create(&db_pool).await?;

                Ok(User::find_by_id(&conn.user_id).one(&db_pool).await?)
            } else {
                Err(Error::Unauthorized)
            }
        }
    }?;

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

        return Ok(HttpResponse::Ok()
            .append_header(("Location", client_url)) // TODO(@Xenfo): Change this to the MFA page
            .finish());
    }

    let redirect_url = session
        .get::<String>(&SessionKey::Redirect.to_string())
        .map_err(|_| {
            Error::InternalServerError("An error occurred while getting the session".into())
        })?
        .map(|url| format!("{}/{}", client_url, url))
        .unwrap_or(format!("{}/dashboard", client_url));

    let auth = auth::authenticate(&db_pool, &config.lock().await.clone(), &user).await?;
    Ok(HttpResponse::Found()
        .cookie(auth.cookie.clone())
        .cookie(
            Cookie::build("isLoggedIn", true.to_string())
                .secure(true)
                .http_only(true)
                .path("/")
                .expires(auth.cookie.expires().unwrap())
                .finish(),
        )
        .append_header(("Location", redirect_url))
        .finish())
}
