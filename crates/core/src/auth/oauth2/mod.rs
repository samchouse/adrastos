use std::collections::HashMap;

use actix_web::web;
use deadpool_redis::redis;
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    reqwest::async_http_client,
    url::Url,
    AuthUrl, AuthorizationCode, AuthorizationRequest, ClientId, ClientSecret, CsrfToken,
    EmptyExtraTokenFields, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    StandardTokenResponse, TokenResponse, TokenUrl,
};
use serde::de::DeserializeOwned;

use crate::config::{self, ConfigKey};

use self::providers::{OAuth2Provider, OAuth2ProviderInfo, OAuth2User, OAuth2UserMethods};

pub mod providers;

#[derive(Clone, Default)]
pub struct OAuth2(HashMap<OAuth2Provider, BasicClient>);

struct ClientInfo {
    client_id: String,
    client_secret: String,
    auth_url: String,
    token_url: String,
}

pub struct OAuth2LoginInfo {
    pub session_csrf_token: String,
    pub params_csrf_token: String,
    pub auth_code: String,
}

trait AddRequiredScopes {
    fn add_required_scopes(self, info: &OAuth2ProviderInfo) -> Self;
}

impl AddRequiredScopes for AuthorizationRequest<'_> {
    fn add_required_scopes(self, info: &OAuth2ProviderInfo) -> Self {
        self.add_scopes(info.scopes.iter().map(|s| Scope::new(s.into())))
    }
}

impl OAuth2 {
    fn providers() -> Vec<OAuth2Provider> {
        vec![
            OAuth2Provider::Google,
            OAuth2Provider::Facebook,
            OAuth2Provider::GitHub,
            OAuth2Provider::Twitter,
            OAuth2Provider::Discord,
        ]
    }

    fn create_client(&mut self, provider: &OAuth2Provider, server_url: &str, info: ClientInfo) {
        let client_id = ClientId::new(info.client_id);
        let client_secret = ClientSecret::new(info.client_secret);
        let auth_url = AuthUrl::new(info.auth_url).unwrap();
        let token_url = TokenUrl::new(info.token_url).unwrap();

        self.0.insert(
            provider.clone(),
            BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
                .set_redirect_uri(
                    RedirectUrl::new(format!(
                        "https://{server_url}/auth/oauth2/callback?provider={provider}"
                    ))
                    .unwrap(),
                ),
        );
    }

    pub fn new(config: config::Config) -> Self {
        let mut oauth2 = Self::default();

        Self::providers().iter().for_each(|provider| {
            let info = provider.info();

            if let Ok(Some(client_id)) = config.get(info.client_info.0.clone()) && let Ok(Some(client_secret)) = config.get(info.client_info.1.clone()) {
                oauth2.create_client(provider, config.get(ConfigKey::ServerUrl).unwrap().unwrap().as_ref(), ClientInfo {
                    client_id,
                    client_secret,
                    auth_url: info.auth_url,
                    token_url: info.token_url,
                });
            }
        });

        oauth2
    }

    pub async fn initialize_login(
        &self,
        provider: OAuth2Provider,
        redis_pool: web::Data<deadpool_redis::Pool>,
    ) -> Result<(Url, CsrfToken), String> {
        let client = self.0.get(&provider).ok_or("Invalid OAuth provider")?;

        let (code_challenge, code_verifier) = PkceCodeChallenge::new_random_sha256();

        let (authorize_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(code_challenge)
            .add_required_scopes(&provider.info())
            .url();

        let mut conn = redis_pool.get().await.unwrap();
        let _: String = redis::cmd("SETEX")
            .arg(format!("oauth:code_verifier:{}", csrf_token.secret()))
            .arg(60 * 10)
            .arg(code_verifier.secret())
            .query_async(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok((authorize_url, csrf_token))
    }

    pub async fn confirm_login(
        &self,
        provider: OAuth2Provider,
        redis_pool: web::Data<deadpool_redis::Pool>,
        security_info: OAuth2LoginInfo,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>, String> {
        let client = self.0.get(&provider).ok_or("Invalid OAuth provider")?;

        let session_csrf_token = CsrfToken::new(security_info.session_csrf_token);
        let params_csrf_token = CsrfToken::new(security_info.params_csrf_token);

        if session_csrf_token.secret().is_empty()
            || params_csrf_token.secret().is_empty()
            || session_csrf_token.secret() != params_csrf_token.secret()
        {
            return Err("Invalid CSRF token".into());
        }

        let mut conn = redis_pool.get().await.unwrap();
        let code_verifier = redis::cmd("GETDEL")
            .arg(format!(
                "oauth:code_verifier:{}",
                session_csrf_token.secret()
            ))
            .query_async(&mut conn)
            .await
            .map_err(|_| "Error getting Redis value")?;

        let code_verifier = PkceCodeVerifier::new(code_verifier);

        let token = client
            .exchange_code(AuthorizationCode::new(security_info.auth_code))
            .set_pkce_verifier(code_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|_| "Unable to get the token from the OAuth provider")?;

        let token_scopes = token
            .scopes()
            .unwrap()
            .iter()
            .map(|scope| scope.to_string())
            .collect::<Vec<String>>();
        if !provider
            .info()
            .scopes
            .iter()
            .all(|scope| token_scopes.contains(scope))
        {
            return Err("Invalid scopes".into());
        }

        Ok(token)
    }

    pub async fn fetch_user<T: OAuth2UserMethods + DeserializeOwned>(
        &self,
        token: &StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    ) -> Result<OAuth2User, String> {
        let client = reqwest::Client::new();

        let user_info = client
            .get(T::get_user_info_url())
            .bearer_auth(token.access_token().secret())
            .send()
            .await
            .map_err(|error| error.to_string())?
            .json::<T>()
            .await
            .map_err(|error| error.to_string())?;

        Ok(OAuth2User {
            id: user_info.get_id(),
            email: user_info.get_email(),
        })
    }
}
