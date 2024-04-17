use adrastos_core::{
    auth::{
        self,
        oauth2::{providers::OAuth2Provider, OAuth2LoginInfo},
    },
    entities,
    error::Error,
    id::Id,
};
use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use serde::Deserialize;
use tower_sessions::Session;
use tracing::error;

use crate::{
    middleware::extractors::{AnyUser, Config, Database, OAuth2},
    session::SessionKey,
    state::AppState,
};

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

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
}

pub async fn login(
    session: Session,
    user: Option<AnyUser>,
    OAuth2(oauth2): OAuth2,
    Query(params): Query<LoginParams>,
    State(AppState { redis_pool, .. }): State<AppState>,
) -> Result<impl IntoResponse, Error> {
    let provider = OAuth2Provider::try_from(params.provider.as_str())
        .map_err(|_| Error::BadRequest("An invalid provider was provided".into()))?;

    let (auth_url, csrf_token) = oauth2
        .initialize_login(provider, &redis_pool)
        .await
        .map_err(|_| Error::InternalServerError("Unable to initialize the OAuth login".into()))?;

    session
        .insert(
            &SessionKey::CsrfToken.to_string(),
            csrf_token.secret().to_string(),
        )
        .await
        .map_err(|_| {
            Error::InternalServerError("Unable to insert CSRF token into session".to_string())
        })?;

    if let Some(AnyUser(user)) = user {
        session
            .insert(&SessionKey::UserId.to_string(), user.id)
            .await
            .map_err(|_| {
                Error::InternalServerError("Unable to insert user ID into session".to_string())
            })?;
    }

    if let Some(to) = params.to.clone() {
        session
            .insert(&SessionKey::Redirect.to_string(), to)
            .await
            .map_err(|_| {
                Error::InternalServerError("Unable to insert redirect URL into session".to_string())
            })?;
    }

    Ok((
        StatusCode::FOUND,
        [(header::LOCATION, auth_url.to_string())],
    ))
}

pub async fn callback(
    jar: CookieJar,
    session: Session,
    Config(config): Config,
    OAuth2(oauth2): OAuth2,
    Database(db): Database,
    Query(params): Query<CallbackParams>,
    State(AppState { redis_pool, .. }): State<AppState>,
) -> Result<impl IntoResponse, Error> {
    let client_url = config.client_url.clone();

    let provider = OAuth2Provider::try_from(params.provider.as_str())
        .map_err(|_| Error::BadRequest("An invalid provider was provided".into()))?;

    let Ok(Some(session_csrf_token)) = session
        .get::<String>(&SessionKey::CsrfToken.to_string())
        .await
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

    let connection = entities::Connection::find()
        .by_provider(provider.to_string())
        .by_provider_id(oauth2_user.id.clone())
        .one(&db)
        .await;

    let user = match connection {
        Ok(conn) => Ok(entities::UserType::from(&db)
            .find_by_id(&conn.user_id)
            .one()
            .await?),
        Err(..) => {
            if let Ok(Some(user_id)) = session.get::<String>(&SessionKey::UserId.to_string()).await
            {
                let conn = entities::Connection {
                    id: Id::new().to_string(),
                    provider: provider.to_string(),
                    provider_id: oauth2_user.id.clone(),
                    user_id,
                    created_at: Utc::now(),
                    updated_at: None,
                };

                conn.create(&db).await?;

                Ok(entities::UserType::from(&db)
                    .find_by_id(&conn.user_id)
                    .one()
                    .await?)
            } else {
                Err(Error::Unauthorized)
            }
        }
    }?;

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

        return Ok([(header::LOCATION, client_url)].into_response()); // TODO(@Xenfo): Change this to the MFA page
    }

    let redirect_url = session
        .get::<String>(&SessionKey::Redirect.to_string())
        .await
        .map_err(|_| {
            Error::InternalServerError("An error occurred while getting the session".into())
        })?
        .map(|url| format!("{}{}", client_url, url))
        .unwrap_or(format!("{}/dashboard", client_url));

    let (_, jar) = auth::authenticate(&db, &config, &user, jar).await?;
    Ok((StatusCode::FOUND, [(header::LOCATION, redirect_url)], jar).into_response())
}
