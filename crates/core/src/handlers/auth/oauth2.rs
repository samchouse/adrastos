use actix_session::Session;
use actix_web::{get, http::header, web, HttpResponse, Responder};
use oauth2::TokenResponse;
use serde::Deserialize;

use crate::{
    auth::oauth2::{OAuth2, OAuth2LoginInfo, OAuth2Provider},
    handlers::Error,
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

#[get("/auth/oauth2/login")]
pub async fn login(
    oauth2: web::Data<OAuth2>,
    params: web::Query<LoginParams>,
    redis_pool: web::Data<deadpool_redis::Pool>,
    session: Session,
) -> impl Responder {
    let Ok(provider) = OAuth2Provider::try_from(params.provider.as_str()) else {
        return HttpResponse::BadRequest().json(Error {
            message: "An invalid provider was provided".to_string(),
        });
    };

    let (auth_url, csrf_token) = oauth2.initialize_login(provider, redis_pool).await.unwrap();

    session
        .insert("csrf_token", csrf_token.secret().to_string())
        .unwrap();

    HttpResponse::Found()
        .append_header((header::LOCATION, auth_url.to_string()))
        .finish()
}

#[get("/auth/oauth2/callback")]
pub async fn callback(
    oauth: web::Data<OAuth2>,
    params: web::Query<CallbackParams>,
    redis_pool: web::Data<deadpool_redis::Pool>,
    session: Session,
) -> impl Responder {
    let Ok(provider) = OAuth2Provider::try_from(params.provider.as_str()) else {
        return HttpResponse::BadRequest().json(Error {
            message: "An invalid provider was provided".to_string(),
        });
    };

    let Ok(Some(session_csrf_token)) = session.get::<String>("csrf_token") else {
        return HttpResponse::BadRequest().json(Error {
            message: "The request is missing a session CSRF Token".to_string(),
        });
    };

    let token = oauth
        .confirm_login(
            provider,
            redis_pool,
            OAuth2LoginInfo {
                session_csrf_token,
                params_csrf_token: params.state.to_string(),
                auth_code: params.code.to_string(),
            },
        )
        .await;
    let Ok(token) = token else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Unable to get the token from the OAuth provider".to_string(),
        });
    };

    HttpResponse::Ok().body(token.access_token().secret().to_string())
}
