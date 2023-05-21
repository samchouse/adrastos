use std::collections::HashMap;

use actix_web::{
    cookie::{time::OffsetDateTime, Cookie, Expiration},
    get, web, HttpRequest, HttpResponse, Responder,
};
use chrono::Utc;
use adrastos_core::{
    auth::{self, TokenType},
    config,
    entities::{Mutate, RefreshTokenTree, RefreshTokenTreeIden, User},
    error::Error,
    util,
};
use sea_query::Alias;
use serde_json::{json, Value};

#[utoipa::path(
    get,
    path = "/auth/token/refresh",
    responses(
        (status = 200, description = ""),
    )
)]
#[get("/token/refresh")]
pub async fn refresh(
    req: HttpRequest,
    config: web::Data<config::Config>,
    db_pool: web::Data<deadpool_postgres::Pool>,
) -> actix_web::Result<impl Responder, Error> {
    let refresh_token = auth::TokenType::verify(&config, util::get_refresh_token(&req)?)?;
    if refresh_token.token_type != TokenType::Refresh {
        return Err(Error::Forbidden("Not a refresh token".into()));
    }

    let user = User::select()
        .by_id(&refresh_token.claims.sub)
        .join::<RefreshTokenTree>(Alias::new(RefreshTokenTreeIden::UserId.to_string()))
        .finish(&db_pool)
        .await?;

    let refresh_token_tree = &user
        .refresh_token_trees
        .clone()
        .ok_or_else(|| Error::Forbidden("Refresh token tree is invalid".into()))?
        .into_iter()
        .find(|tree| tree.tokens.contains(&refresh_token.claims.jti))
        .ok_or_else(|| Error::Forbidden("Refresh token tree is invalid".into()))?;

    let last_token = refresh_token_tree
        .tokens
        .last()
        .ok_or_else(|| Error::Forbidden("Refresh token tree is invalid".into()))?;

    if refresh_token_tree.inactive_at < Utc::now() || refresh_token_tree.expires_at < Utc::now() {
        return Err(Error::Forbidden("Refresh token tree has expired".into()));
    } else if refresh_token.claims.jti.clone().as_str() != last_token.as_str() {
        refresh_token_tree.delete(&db_pool).await?;

        return Err(Error::Forbidden("Refresh token is invalid".into()));
    }

    let access_token = TokenType::Access.sign(&config, &user)?;
    let refresh_token = TokenType::Access.sign(&config, &user)?;

    let cookie_expiration = OffsetDateTime::from_unix_timestamp(
        refresh_token.expires_at.timestamp(),
    )
    .map_err(|_| {
        Error::InternalServerError("An error occurred while parsing the cookie expiration".into())
    })?;

    let mut tokens = refresh_token_tree.tokens.clone();
    tokens.push(refresh_token.claims.jti.clone());

    refresh_token_tree
        .update(
            &db_pool,
            &HashMap::from([(
                RefreshTokenTreeIden::Tokens.to_string(),
                Value::from(tokens),
            )]),
        )
        .await?;

    Ok(HttpResponse::Ok()
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
        })))
}
