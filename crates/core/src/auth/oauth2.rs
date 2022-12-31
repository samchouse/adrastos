use std::collections::HashMap;

use actix_web::web;
use deadpool_redis::redis;
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    reqwest::async_http_client,
    url::Url,
    AuthUrl, AuthorizationCode, AuthorizationRequest, ClientId, ClientSecret, CsrfToken,
    EmptyExtraTokenFields, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    StandardTokenResponse, TokenUrl,
};

use crate::config::{self, ConfigKey};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum OAuth2Provider {
    Google,
    Facebook,
    GitHub,
    Twitter,
    Discord,
}

impl ToString for OAuth2Provider {
    fn to_string(&self) -> String {
        match self {
            OAuth2Provider::Google => "google".to_string(),
            OAuth2Provider::Facebook => "facebook".to_string(),
            OAuth2Provider::GitHub => "github".to_string(),
            OAuth2Provider::Twitter => "twitter".to_string(),
            OAuth2Provider::Discord => "discord".to_string(),
        }
    }
}

impl TryFrom<&str> for OAuth2Provider {
    type Error = ();

    fn try_from(provider: &str) -> Result<Self, Self::Error> {
        match provider {
            "google" => Ok(OAuth2Provider::Google),
            "facebook" => Ok(OAuth2Provider::Facebook),
            "github" => Ok(OAuth2Provider::GitHub),
            "twitter" => Ok(OAuth2Provider::Twitter),
            "discord" => Ok(OAuth2Provider::Discord),
            &_ => Err(()),
        }
    }
}

struct OAuth2ProviderOptions {
    provider: OAuth2Provider,
    client_info: (ConfigKey, ConfigKey),
    auth_url: &'static str,
    token_url: &'static str,
}

#[derive(Clone)]
pub struct OAuth2(HashMap<OAuth2Provider, BasicClient>);

struct ClientOptions<'a> {
    client_id: String,
    client_secret: String,
    auth_url: &'a str,
    token_url: &'a str,
}

pub struct OAuth2LoginInfo {
    pub session_csrf_token: String,
    pub params_csrf_token: String,
    pub auth_code: String,
}

trait AddPresetScopes {
    fn add_preset_scopes(self, provider: &OAuth2Provider) -> Self;
}

impl AddPresetScopes for AuthorizationRequest<'_> {
    fn add_preset_scopes(self, provider: &OAuth2Provider) -> Self {
        match provider {
            OAuth2Provider::Google => self.add_scopes([
                Scope::new("https://www.googleapis.com/auth/userinfo.email".to_string()),
                Scope::new("https://www.googleapis.com/auth/userinfo.profile".to_string()),
            ]),
            OAuth2Provider::Facebook => self.add_scopes([
                Scope::new("email".to_string()),
                Scope::new("public_profile".to_string()),
            ]),
            OAuth2Provider::GitHub => self.add_scopes([
                Scope::new("read:user".to_string()),
                Scope::new("user:email".to_string()),
            ]),
            OAuth2Provider::Twitter => self.add_scopes([Scope::new("users.read".to_string())]),
            OAuth2Provider::Discord => self.add_scopes([
                Scope::new("identify".to_string()),
                Scope::new("email".to_string()),
            ]),
        }
    }
}

impl OAuth2 {
    fn options() -> Vec<OAuth2ProviderOptions> {
        vec![
            OAuth2ProviderOptions {
                provider: OAuth2Provider::Google,
                client_info: (ConfigKey::GoogleClientId, ConfigKey::GoogleClientSecret),
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth",
                token_url: "https://oauth2.googleapis.com/token",
            },
            OAuth2ProviderOptions {
                provider: OAuth2Provider::Facebook,
                client_info: (ConfigKey::FacebookClientId, ConfigKey::FacebookClientSecret),
                auth_url: "https://www.facebook.com/v15.0/dialog/oauth",
                token_url: "https://graph.facebook.com/v15.0/oauth/access_token",
            },
            OAuth2ProviderOptions {
                provider: OAuth2Provider::GitHub,
                client_info: (ConfigKey::GitHubClientId, ConfigKey::GitHubClientSecret),
                auth_url: "https://github.com/login/oauth/authorize",
                token_url: "https://github.com/login/oauth/access_token",
            },
            OAuth2ProviderOptions {
                provider: OAuth2Provider::Twitter,
                client_info: (ConfigKey::TwitterClientId, ConfigKey::TwitterClientSecret),
                auth_url: "https://twitter.com/i/oauth2/authorize",
                token_url: "https://api.twitter.com/2/oauth2/token",
            },
            OAuth2ProviderOptions {
                provider: OAuth2Provider::Discord,
                client_info: (ConfigKey::DiscordClientId, ConfigKey::DiscordClientSecret),
                auth_url: "https://discord.com/oauth2/authorize",
                token_url: "https://discord.com/api/oauth2/token",
            },
        ]
    }

    fn create_client(&mut self, provider: OAuth2Provider, url: &str, options: ClientOptions) {
        let client_id = ClientId::new(options.client_id);
        let client_secret = ClientSecret::new(options.client_secret);
        let auth_url = AuthUrl::new(options.auth_url.to_string()).unwrap();
        let token_url = TokenUrl::new(options.token_url.to_string()).unwrap();

        self.0.insert(
            provider.clone(),
            BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
                .set_redirect_uri(
                    RedirectUrl::new(format!(
                        "https://{}/auth/oauth2/callback?provider={}",
                        url,
                        provider.to_string()
                    ))
                    .unwrap(),
                ),
        );
    }

    pub fn new(config: config::Config) -> Self {
        let mut oauth2 = Self::default();

        Self::options().iter().for_each(|options| {
            if let Some(client_id) = config.get(options.client_info.0.clone()) && let Some(client_secret) = config.get(options.client_info.1.clone()) {
                oauth2.create_client(options.provider.clone(), config.get(ConfigKey::Url).unwrap().as_ref(), ClientOptions {
                    client_id,
                    client_secret,
                    auth_url: options.auth_url,
                    token_url: options.token_url,
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
            .add_preset_scopes(&provider)
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
            return Err("Invalid CSRF token".to_string());
        }

        let mut conn = redis_pool.get().await.unwrap();
        let code_verifier = redis::cmd("GETDEL")
            .arg(format!(
                "oauth:code_verifier:{}",
                session_csrf_token.secret()
            ))
            .query_async(&mut conn)
            .await
            .map_err(|error| error.to_string())?;

        let code_verifier = PkceCodeVerifier::new(code_verifier);

        let token = client
            .exchange_code(AuthorizationCode::new(security_info.auth_code))
            .set_pkce_verifier(code_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|error| error.to_string())?;

        Ok(token.clone())
    }
}

impl Default for OAuth2 {
    fn default() -> Self {
        OAuth2(HashMap::new())
    }
}
