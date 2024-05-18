use std::{collections::HashMap, fmt::Debug};

use chrono::Duration;
use fred::{clients::RedisPool, interfaces::KeysInterface, types::Expiration};
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    reqwest::async_http_client,
    url::Url,
    AuthUrl, AuthorizationCode, AuthorizationRequest, ClientId, ClientSecret, CsrfToken,
    EmptyExtraTokenFields, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    StandardTokenResponse, TokenResponse, TokenUrl,
};
use secrecy::ExposeSecret;
use serde::{de::DeserializeOwned, Deserialize};

use crate::{config, db::redis};

use self::providers::{OAuth2Provider, OAuth2ProviderInfo, OAuth2User, OAuth2UserMethods};

pub mod providers;

#[derive(Clone, Default)]
pub struct OAuth2(HashMap<OAuth2Provider, BasicClient>, config::Config);

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

#[derive(Deserialize, Debug)]
struct FacebookTokenDebugResponse {
    data: FacebookTokenDebugResponseData,
}

#[derive(Deserialize, Debug)]
struct FacebookTokenDebugResponseData {
    scopes: Vec<String>,
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
                        "https://{server_url}/api/auth/oauth2/callback?provider={provider}"
                    ))
                    .unwrap(),
                ),
        );
    }

    pub fn new(config: &config::Config) -> Self {
        let mut oauth2 = Self {
            1: config.clone(),
            ..Self::default()
        };

        Self::providers().iter().for_each(|provider| {
            let info = provider.info();

            let (client_id, client_secret) = match provider {
                OAuth2Provider::Google => (&config.google_client_id, &config.google_client_secret),
                OAuth2Provider::Facebook => {
                    (&config.facebook_client_id, &config.facebook_client_secret)
                }
                OAuth2Provider::GitHub => (&config.github_client_id, &config.github_client_secret),
                OAuth2Provider::Twitter => {
                    (&config.twitter_client_id, &config.twitter_client_secret)
                }
                OAuth2Provider::Discord => {
                    (&config.discord_client_id, &config.discord_client_secret)
                }
            };

            if let Some(client_id) = client_id.to_owned() {
                if let Some(client_secret) = client_secret.to_owned() {
                    oauth2.create_client(
                        provider,
                        &config.server_url,
                        ClientInfo {
                            client_id,
                            client_secret: client_secret.expose_secret().to_string(),
                            auth_url: info.auth_url,
                            token_url: info.token_url,
                        },
                    );
                }
            }
        });

        oauth2
    }

    pub async fn initialize_login(
        &self,
        provider: OAuth2Provider,
        redis_pool: &RedisPool,
    ) -> Result<(Url, CsrfToken), String> {
        let client = self.0.get(&provider).ok_or("Invalid OAuth provider")?;

        let (code_challenge, code_verifier) = PkceCodeChallenge::new_random_sha256();

        let (authorize_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(code_challenge)
            .add_required_scopes(&provider.info())
            .url();

        redis_pool
            .set(
                redis::build_key(
                    &self.1,
                    format!("oauth:code_verifier:{}", csrf_token.secret()),
                ),
                code_verifier.secret(),
                Some(Expiration::EX(Duration::minutes(10).num_seconds())),
                None,
                false,
            )
            .await
            .map_err(|e| e.to_string())?;

        Ok((authorize_url, csrf_token))
    }

    pub async fn confirm_login(
        &self,
        provider: OAuth2Provider,
        config: &config::Config,
        redis_pool: &RedisPool,
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

        let code_verifier = redis_pool
            .getdel(redis::build_key(
                &self.1,
                format!("oauth:code_verifier:{}", session_csrf_token.secret()),
            ))
            .await
            .map_err(|_| "Error getting Redis value")?;

        let code_verifier = PkceCodeVerifier::new(code_verifier);

        let token = client
            .exchange_code(AuthorizationCode::new(security_info.auth_code))
            .set_pkce_verifier(code_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|_| "Unable to get the token from the OAuth provider")?;

        let token_scopes = match provider {
            OAuth2Provider::Facebook => {
                let r_client = reqwest::Client::new();

                let debug_result = r_client
                    .get(format!(
                        "https://graph.facebook.com/debug_token?input_token={}&access_token={}|{}",
                        token.access_token().secret(),
                        client.client_id().as_str(),
                        config
                            .facebook_client_secret
                            .as_ref()
                            .unwrap()
                            .expose_secret(),
                    ))
                    .send()
                    .await
                    .map_err(|_| "Unable to get the app token from Facebook")?
                    .json::<FacebookTokenDebugResponse>()
                    .await
                    .map_err(|_| "Unable to get the app token from Facebook")?;

                debug_result.data.scopes
            }
            _ => token
                .scopes()
                .unwrap()
                .iter()
                .map(|scope| scope.to_string())
                .collect::<Vec<String>>(),
        };
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

    pub async fn fetch_user<T: OAuth2UserMethods + DeserializeOwned + Debug>(
        &self,
        token: &StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    ) -> Result<OAuth2User, String> {
        let client = reqwest::Client::new();

        let user_info = client
            .get(T::get_user_info_url())
            .bearer_auth(token.access_token().secret())
            .header("User-Agent", "Adrastos")
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
