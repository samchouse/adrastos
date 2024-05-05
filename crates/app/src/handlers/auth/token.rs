use actix_web::{
    cookie::{time::OffsetDateTime, Cookie, Expiration, SameSite},
    get, HttpRequest, HttpResponse, Responder,
};
use adrastos_core::{
    auth::{self, TokenType},
    db::postgres::Database,
    entities::{AnyUserJoin, UserType},
    error::Error,
    util,
};
use chrono::Utc;

use crate::middleware::config::Config;

#[get("/token/refresh")]
#[tracing::instrument(skip(config, req, db))]
pub async fn refresh(
    db: Database,
    config: Config,
    req: HttpRequest,
) -> actix_web::Result<impl Responder, Error> {
    let refresh_token = auth::TokenType::verify(
        &config.clone(),
        util::get_auth_cookies(&req)?.refresh_token.value().into(),
    )?;
    if refresh_token.token_type != TokenType::Refresh {
        return Err(Error::Forbidden("Not a refresh token".into()));
    }

    let user = UserType::from(&db)
        .find_by_id(&refresh_token.claims.sub)
        .join(AnyUserJoin::RefreshTokenTrees)
        .one()
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
        refresh_token_tree.delete(&db).await?;

        return Err(Error::Forbidden("Refresh token is invalid".into()));
    }

    let access_token = TokenType::Access.sign(&config.clone(), &user)?;
    let refresh_token = TokenType::Refresh.sign(&config.clone(), &user)?;

    let cookie_expiration = OffsetDateTime::from_unix_timestamp(
        refresh_token.expires_at.timestamp(),
    )
    .map_err(|_| {
        Error::InternalServerError("An error occurred while parsing the cookie expiration".into())
    })?;

    let mut tokens = refresh_token_tree.tokens.clone();
    tokens.push(refresh_token.claims.jti.clone());

    refresh_token_tree.update(&db, tokens).await?;

    Ok(HttpResponse::Ok()
        .cookie(
            Cookie::build("refreshToken", refresh_token.token)
                .secure(true)
                .http_only(true)
                .same_site(SameSite::None)
                .path("/api/auth")
                .expires(Expiration::from(cookie_expiration))
                .finish(),
        )
        .cookie(
            Cookie::build("isLoggedIn", true.to_string())
                .secure(true)
                .http_only(true)
                .same_site(SameSite::None)
                .path("/")
                .expires(Expiration::from(cookie_expiration))
                .finish(),
        )
        .json(access_token.clone().token))
}
