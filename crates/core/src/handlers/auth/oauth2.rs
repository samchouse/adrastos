use actix_session::Session;
use actix_web::{get, http::header, web, HttpResponse, Responder};
use oauth2::TokenResponse;
use sea_query::{Expr, Query};
use serde::Deserialize;

use crate::{
    auth::oauth2::{providers::OAuth2Provider, OAuth2, OAuth2LoginInfo},
    entities::{Connection, ConnectionIden, Queries},
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

    let Ok((auth_url, csrf_token)) = oauth2.initialize_login(provider, redis_pool).await else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Unable to initialize the OAuth login".to_string(),
        });
    };

    session
        .insert("csrf_token", csrf_token.secret().to_string())
        .unwrap();

    HttpResponse::Found()
        .append_header((header::LOCATION, auth_url.to_string()))
        .finish()
}

#[get("/auth/oauth2/callback")]
pub async fn callback(
    oauth2: web::Data<OAuth2>,
    params: web::Query<CallbackParams>,
    db_pool: web::Data<deadpool_postgres::Pool>,
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

    match oauth2
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
    {
        Ok(token) => {
            let Ok(oauth2_user) = provider.fetch_user(&oauth2, &token).await else {
                return HttpResponse::InternalServerError().json(Error {
                    message: "Unable to fetch the user from the OAuth provider".to_string(),
                });
            };

            let Ok(rows) = db_pool.get().await.unwrap().query(Connection::query_select(Query::select().and_where(Expr::col(ConnectionIden::Provider).like(provider.clone().to_string())).and_where(Expr::col(ConnectionIden::ProviderId).like(oauth2_user.id)).limit(1)).as_str(), &[]).await else {
                return HttpResponse::InternalServerError().json(Error {
                    message: "An error occurred while fetching the connection".to_string(),
                });
            };

            let Some(row) = rows.iter().next() else {
                return HttpResponse::BadRequest().json(Error {
                    message: "No connection was found".to_string(),
                });
            };

            let connection = Connection::from(row);

            HttpResponse::Ok().body(token.access_token().secret().to_string()) // TODO(@Xenfo): don't send the token back
        }
        Err(err) => HttpResponse::InternalServerError().json(Error { message: err }),
    }
}
