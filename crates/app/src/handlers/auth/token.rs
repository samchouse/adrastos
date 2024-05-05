use adrastos_core::{
    auth::{self, TokenType},
    entities,
    error::Error,
    util,
};
use axum::{response::IntoResponse, routing::get, Json, Router};
use axum_extra::extract::CookieJar;
use chrono::Utc;

use crate::{
    middleware::extractors::{Config, Database},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/refresh", get(refresh))
}

#[tracing::instrument(skip(config, jar, db))]
pub async fn refresh(
    jar: CookieJar,
    Config(config): Config,
    Database(db): Database,
) -> Result<impl IntoResponse, Error> {
    let refresh_token = auth::TokenType::verify(
        &config.clone(),
        util::get_auth_cookies(&jar)?.refresh_token.value().into(),
    )?;
    if refresh_token.token_type != TokenType::Refresh {
        return Err(Error::Forbidden("Not a refresh token".into()));
    }

    let user = entities::UserType::from(&db)
        .find_by_id(&refresh_token.claims.sub)
        .join(entities::AnyUserJoin::RefreshTokenTrees)
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

    let mut tokens = refresh_token_tree.tokens.clone();
    tokens.push(refresh_token.claims.jti.clone());
    refresh_token_tree.update(&db, tokens).await?;

    Ok((
        auth::create_auth_cookies(refresh_token, jar)?,
        Json(access_token.clone().token),
    ))
}
