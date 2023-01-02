use oauth2::{basic::BasicTokenType, EmptyExtraTokenFields, StandardTokenResponse};
use serde::{Deserialize, Serialize};

use crate::config::ConfigKey;

use super::OAuth2;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum OAuth2Provider {
    Google,
    Facebook,
    GitHub,
    Twitter,
    Discord,
}

impl OAuth2Provider {
    pub fn info(&self) -> OAuth2ProviderInfo {
        match self {
            OAuth2Provider::Google => OAuth2ProviderInfo {
                client_info: (ConfigKey::GoogleClientId, ConfigKey::GoogleClientSecret),
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                token_url: "https://oauth2.googleapis.com/token".to_string(),
                scopes: vec![
                    "https://www.googleapis.com/auth/userinfo.email".to_string(),
                    "https://www.googleapis.com/auth/userinfo.profile".to_string(),
                ],
            },
            OAuth2Provider::Facebook => OAuth2ProviderInfo {
                client_info: (ConfigKey::FacebookClientId, ConfigKey::FacebookClientSecret),
                auth_url: "https://www.facebook.com/v15.0/dialog/oauth".to_string(),
                token_url: "https://graph.facebook.com/v15.0/oauth/access_token".to_string(),
                scopes: vec!["public_profile".to_string()],
            },
            OAuth2Provider::GitHub => OAuth2ProviderInfo {
                client_info: (ConfigKey::GitHubClientId, ConfigKey::GitHubClientSecret),
                auth_url: "https://github.com/login/oauth/authorize".to_string(),
                token_url: "https://github.com/login/oauth/access_token".to_string(),
                scopes: vec!["read:user".to_string()],
            },
            OAuth2Provider::Twitter => OAuth2ProviderInfo {
                client_info: (ConfigKey::TwitterClientId, ConfigKey::TwitterClientSecret),
                auth_url: "https://twitter.com/i/oauth2/authorize".to_string(),
                token_url: "https://api.twitter.com/2/oauth2/token".to_string(),
                scopes: vec!["users.read".to_string()],
            },
            OAuth2Provider::Discord => OAuth2ProviderInfo {
                client_info: (ConfigKey::DiscordClientId, ConfigKey::DiscordClientSecret),
                auth_url: "https://discord.com/oauth2/authorize".to_string(),
                token_url: "https://discord.com/api/oauth2/token".to_string(),
                scopes: vec!["identify".to_string(), "email".to_string()],
            },
        }
    }

    pub async fn fetch_user(
        &self,
        oauth2: &OAuth2,
        token: &StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    ) -> Result<OAuth2User, String> {
        match self {
            OAuth2Provider::Google => oauth2.fetch_user::<GoogleUser>(token).await,
            OAuth2Provider::Facebook => oauth2.fetch_user::<FacebookUser>(token).await,
            OAuth2Provider::GitHub => oauth2.fetch_user::<GitHubUser>(token).await,
            OAuth2Provider::Twitter => oauth2.fetch_user::<TwitterUser>(token).await,
            OAuth2Provider::Discord => oauth2.fetch_user::<DiscordUser>(token).await,
        }
    }
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

pub struct OAuth2User {
    pub id: String,
    pub email: Option<String>,
}

pub struct OAuth2ProviderInfo {
    pub client_info: (ConfigKey, ConfigKey),
    pub auth_url: String,
    pub token_url: String,
    pub scopes: Vec<String>,
}

#[derive(Deserialize)]
struct GoogleUser {
    sub: String,
    email: String,
    email_verified: bool,
}

#[derive(Deserialize)]
struct FacebookUser {
    id: String,
}

#[derive(Deserialize)]
struct GitHubUser {
    id: String,
}

#[derive(Deserialize)]
struct TwitterUser {
    id: String,
}

#[derive(Deserialize)]
struct DiscordUser {
    id: String,
    email: String,
    verified: bool,
}

pub trait OAuth2UserMethods {
    fn get_user_info_url() -> String;
    fn get_id(&self) -> String;

    fn get_email(&self) -> Option<String> {
        None
    }
}

impl OAuth2UserMethods for GoogleUser {
    fn get_user_info_url() -> String {
        "https://openidconnect.googleapis.com/v1/userinfo".to_string()
    }

    fn get_id(&self) -> String {
        self.sub.clone()
    }

    fn get_email(&self) -> Option<String> {
        if self.email_verified {
            Some(self.email.clone())
        } else {
            None
        }
    }
}

impl OAuth2UserMethods for FacebookUser {
    fn get_user_info_url() -> String {
        "https://graph.facebook.com/me?fields=id,email".to_string()
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl OAuth2UserMethods for GitHubUser {
    fn get_user_info_url() -> String {
        "https://api.github.com/user".to_string()
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl OAuth2UserMethods for TwitterUser {
    fn get_user_info_url() -> String {
        "https://api.twitter.com/2/users/me".to_string()
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl OAuth2UserMethods for DiscordUser {
    fn get_user_info_url() -> String {
        "https://discord.com/api/users/@me".to_string()
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_email(&self) -> Option<String> {
        if self.verified {
            Some(self.email.clone())
        } else {
            None
        }
    }
}
