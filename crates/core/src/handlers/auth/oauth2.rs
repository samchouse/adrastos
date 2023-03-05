use std::fmt;

use actix_session::Session;
use actix_web::{
    cookie::{time::OffsetDateTime, Cookie, Expiration},
    get,
    http::header::{self, Header},
    web, HttpRequest, HttpResponse, Responder,
};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use chrono::Utc;
use sea_query::Expr;
use serde::Deserialize;

use crate::{
    auth::{
        oauth2::{providers::OAuth2Provider, OAuth2, OAuth2LoginInfo},
        TokenType,
    },
    config::{self, ConfigKey},
    entities::{Connection, ConnectionIden, Mutate, User, UserIden},
    handlers::Error,
    id::Id,
};

#[derive(Deserialize)]
pub struct LoginParams {
    provider: String,
}

#[derive(Deserialize)]
pub struct CallbackParams {
    provider: String,
    state: String,
    code: String,
}

enum SessionKey {
    UserId,
    CsrfToken,
}

impl fmt::Display for SessionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            SessionKey::UserId => "user_id",
            SessionKey::CsrfToken => "csrf_token",
        };

        write!(f, "{name}")
    }
}

#[get("/login")]
pub async fn login(
    req: HttpRequest,
    oauth2: web::Data<OAuth2>,
    params: web::Query<LoginParams>,
    config: web::Data<config::Config>,
    redis_pool: web::Data<deadpool_redis::Pool>,
    session: Session,
) -> actix_web::Result<impl Responder, Error> {
    let provider =
        OAuth2Provider::try_from(params.provider.as_str()).map_err(|_| Error::BadRequest {
            message: "An invalid provider was provided".into(),
        })?;

    let (auth_url, csrf_token) = oauth2
        .initialize_login(provider, redis_pool)
        .await
        .map_err(|_| Error::InternalServerError {
            error: "Unable to initialize the OAuth login".into(),
        })?;

    session
        .insert(
            SessionKey::CsrfToken.to_string(),
            csrf_token.secret().to_string(),
        )
        .map_err(|_| Error::InternalServerError {
            error: "Unable to insert CSRF token into session".to_string(),
        })?;

    if let Ok(auth) = Authorization::<Bearer>::parse(&req) {
        if let Ok(access_token) = TokenType::verify(&config, auth.into_scheme().token().into()) {
            session
                .insert(SessionKey::UserId.to_string(), access_token.claims.sub)
                .map_err(|_| Error::InternalServerError {
                    error: "Unable to insert user ID into session".to_string(),
                })?;
        }
    };

    Ok(HttpResponse::Found()
        .append_header((header::LOCATION, auth_url.to_string()))
        .finish())
}

#[get("/callback")]
pub async fn callback(
    config: web::Data<config::Config>,
    oauth2: web::Data<OAuth2>,
    params: web::Query<CallbackParams>,
    db_pool: web::Data<deadpool_postgres::Pool>,
    redis_pool: web::Data<deadpool_redis::Pool>,
    session: Session,
) -> actix_web::Result<impl Responder, Error> {
    let client_url = config.get(ConfigKey::ClientUrl)?;

    let provider =
        OAuth2Provider::try_from(params.provider.as_str()).map_err(|_| Error::BadRequest {
            message: "An invalid provider was provided".into(),
        })?;

    let Ok(Some(session_csrf_token)) = session.get::<String>(&SessionKey::CsrfToken.to_string()) else {
        return Err(Error::BadRequest { message: "The request is missing a session CSRF Token".into() });
    };

    let token = oauth2
        .confirm_login(
            provider.clone(),
            redis_pool,
            OAuth2LoginInfo {
                session_csrf_token,
                params_csrf_token: params.state.to_string(),
                auth_code: params.code.to_string(),
            },
        )
        .await
        .map_err(|err| Error::InternalServerError { error: err })?;

    let oauth2_user =
        provider
            .fetch_user(&oauth2, &token)
            .await
            .map_err(|_| Error::InternalServerError {
                error: "Unable to fetch the user from the OAuth provider".into(),
            })?;

    let connection = Connection::find(
        &db_pool,
        vec![
            Expr::col(ConnectionIden::Provider).eq(&provider.to_string()),
            Expr::col(ConnectionIden::ProviderId).eq(&oauth2_user.id),
        ],
    )
    .await;

    let user = match connection {
        Ok(conn) => Ok(User::find(&db_pool, vec![Expr::col(UserIden::Id).eq(conn.user_id)]).await?),
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

                Ok(User::select().by_id(&conn.user_id).finish(&db_pool).await?)
            } else {
                Err(Error::Unauthorized)
            }
        }
    }?;

    let refresh_token = TokenType::Access.sign(&config, &user)?;

    let cookie_expiration = OffsetDateTime::from_unix_timestamp(
        refresh_token.expires_at.timestamp(),
    )
    .map_err(|_| Error::InternalServerError {
        error: "An error occurred while parsing the cookie expiration".into(),
    })?;

    Ok(HttpResponse::Found()
        .cookie(
            Cookie::build("refreshToken", refresh_token.token)
                .path("/auth")
                .secure(true)
                .http_only(true)
                .expires(Expiration::from(cookie_expiration))
                .finish(),
        )
        .append_header(("Location", client_url))
        .finish())
}
