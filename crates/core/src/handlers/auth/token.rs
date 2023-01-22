use std::collections::HashMap;

use actix_web::{
    cookie::{time::OffsetDateTime, Cookie, Expiration},
    get, web, HttpRequest, HttpResponse, Responder,
};
use chrono::Utc;
use sea_query::{extension::postgres::PgExpr, Expr};
use serde_json::{json, Value};

use crate::{
    auth::{self, TokenType},
    config,
    entities::{Queries, RefreshTokenTree, RefreshTokenTreeIden, User, UserIden},
    handlers::Error,
};

#[get("/auth/token/refresh")]
pub async fn refresh(
    req: HttpRequest,
    config: web::Data<config::Config>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> impl Responder {
    let Ok(cookies) = req.cookies() else {
        return HttpResponse::InternalServerError().json(Error {
            message: "An error occurred while fetching the cookies".to_string(),
        });
    };

    let Some(cookie) = cookies.iter().find(|cookie| cookie.name() == "refreshToken") else {
        return HttpResponse::BadRequest().json(Error {
            message: "No refresh token was found".to_string(),
        });
    };

    let refresh_token =
        auth::TokenType::verify(config.clone(), cookie.value().to_string()).unwrap();

    let result = db_pool
        .get()
        .await
        .unwrap()
        .query(
            RefreshTokenTree::query_select(vec![
                Expr::col(RefreshTokenTreeIden::UserId).eq(refresh_token.claims.sub.clone()),
                Expr::col(RefreshTokenTreeIden::Tokens)
                    .contains(vec![refresh_token.claims.jti.clone()]),
            ])
            .as_str(),
            &[],
        )
        .await;
    let Ok(rows) = result else {
        return HttpResponse::InternalServerError().json(Error {
            message: "An error occurred while fetching the token tree".to_string(),
        });
    };

    let Some(row) = rows.iter().next() else {
        return HttpResponse::BadRequest().json(Error {
            message: "No token tree was found".to_string(),
        });
    };

    let refresh_token_tree = RefreshTokenTree::from(row);

    if refresh_token_tree.expires_at < Utc::now() {
        return HttpResponse::BadRequest().json(Error {
            message: "The refresh token has expired".to_string(),
        });
    }

    let Some(last_token) = refresh_token_tree.tokens.last() else {
        return HttpResponse::BadRequest().json(Error {
            message: "The refresh token is invalid".to_string(),
        });
    };
    if refresh_token.claims.jti.clone().as_str() != last_token.as_str() {
        let result = db_pool
            .get()
            .await
            .unwrap()
            .query(
                RefreshTokenTree::query_delete(
                    Expr::col(RefreshTokenTreeIden::Id).eq(refresh_token_tree.id),
                )
                .as_str(),
                &[],
            )
            .await;
        if result.is_err() {
            return HttpResponse::InternalServerError().json(Error {
                message: "An error occurred while deleting the token tree".to_string(),
            });
        }

        return HttpResponse::BadRequest().json(Error {
            message: "The refresh token is invalid".to_string(),
        });
    }

    let rows = db_pool
        .get()
        .await
        .unwrap()
        .query(
            User::query_select(vec![Expr::col(UserIden::Id).eq(refresh_token.claims.sub)]).as_str(),
            &[],
        )
        .await;
    let Ok(rows) = rows else {
        return HttpResponse::InternalServerError().json(Error {
            message: "An error occurred while fetching the user".to_string(),
        });
    };

    let Some(row) = rows.iter().next() else {
        return HttpResponse::BadRequest().json(Error {
            message: "No user was found".to_string(),
        });
    };

    let user = User::from(row);

    let Ok(access_token) = TokenType::Access.sign(config.clone(), user.clone()) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "An error occurred while signing the access token".to_string(),
        });
    };
    let Ok(refresh_token) = TokenType::Refresh.sign(config, user.clone()) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "An error occurred while signing the refresh token".to_string(),
        });
    };

    let Ok(cookie_expiration) = OffsetDateTime::from_unix_timestamp(refresh_token.expires_at.timestamp()) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "An error occurred while parsing the cookie expiration".to_string(),
        });
    };

    let mut tokens = refresh_token_tree.tokens.clone();
    tokens.push(refresh_token.claims.jti.clone());

    let query = refresh_token_tree.query_update(HashMap::from([(
        RefreshTokenTreeIden::Tokens.to_string(),
        Value::from(tokens),
    )]));
    let Ok(query) = query else {
        return HttpResponse::InternalServerError().json(Error {
            message: "An error occurred while parsing the query".to_string(),
        });
    };

    let result = db_pool
        .get()
        .await
        .unwrap()
        .query(query.as_str(), &[])
        .await;
    if result.is_err() {
        return HttpResponse::InternalServerError().json(Error {
            message: "An error occurred while updating the token tree".to_string(),
        });
    }

    HttpResponse::Ok()
        .cookie(
            Cookie::build("refreshToken", refresh_token.token)
                .path("/auth")
                .secure(true)
                .http_only(true)
                .expires(Expiration::from(cookie_expiration))
                .finish(),
        )
        .json(json!({
            "success": true,
            "accessToken": access_token.clone().token,
        }))
}
