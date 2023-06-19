use std::fmt;

use oauth2::{basic::BasicTokenType, EmptyExtraTokenFields, StandardTokenResponse};
use serde::{Deserialize, Serialize};

use crate::config::ConfigKey;

use super::OAuth2;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
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
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth".into(),
                token_url: "https://oauth2.googleapis.com/token".into(),
                scopes: vec![
                    "https://www.googleapis.com/auth/userinfo.email".into(),
                    "https://www.googleapis.com/auth/userinfo.profile".into(),
                ],
            },
            OAuth2Provider::Facebook => OAuth2ProviderInfo {
                client_info: (ConfigKey::FacebookClientId, ConfigKey::FacebookClientSecret),
                auth_url: "https://www.facebook.com/v15.0/dialog/oauth".into(),
                token_url: "https://graph.facebook.com/v15.0/oauth/access_token".into(),
                scopes: vec!["public_profile".into()],
            },
            OAuth2Provider::GitHub => OAuth2ProviderInfo {
                client_info: (ConfigKey::GitHubClientId, ConfigKey::GitHubClientSecret),
                auth_url: "https://github.com/login/oauth/authorize".into(),
                token_url: "https://github.com/login/oauth/access_token".into(),
                scopes: vec!["read:user".into()],
            },
            OAuth2Provider::Twitter => OAuth2ProviderInfo {
                client_info: (ConfigKey::TwitterClientId, ConfigKey::TwitterClientSecret),
                auth_url: "https://twitter.com/i/oauth2/authorize".into(),
                token_url: "https://api.twitter.com/2/oauth2/token".into(),
                scopes: vec!["users.read".into()],
            },
            OAuth2Provider::Discord => OAuth2ProviderInfo {
                client_info: (ConfigKey::DiscordClientId, ConfigKey::DiscordClientSecret),
                auth_url: "https://discord.com/oauth2/authorize".into(),
                token_url: "https://discord.com/api/oauth2/token".into(),
                scopes: vec!["identify".into(), "email".into()],
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

impl fmt::Display for OAuth2Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            OAuth2Provider::Google => "google",
            OAuth2Provider::Facebook => "facebook",
            OAuth2Provider::GitHub => "github",
            OAuth2Provider::Twitter => "twitter",
            OAuth2Provider::Discord => "discord",
        };

        write!(f, "{name}")
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

#[derive(Deserialize, Debug)]
struct GoogleUser {
    sub: String,
    email: String,
    email_verified: bool,
}

#[derive(Deserialize, Debug)]
struct FacebookUser {
    id: String,
}

#[derive(Deserialize, Debug)]
struct GitHubUser {
    id: u32,
}

#[derive(Deserialize, Debug)]
struct TwitterUser {
    id: String,
}

#[derive(Deserialize, Debug)]
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
        "https://openidconnect.googleapis.com/v1/userinfo".into()
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
        "https://graph.facebook.com/me?fields=id,email".into()
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl OAuth2UserMethods for GitHubUser {
    fn get_user_info_url() -> String {
        "https://api.github.com/user".into()
    }

    fn get_id(&self) -> String {
        self.id.to_string()
    }
}

impl OAuth2UserMethods for TwitterUser {
    fn get_user_info_url() -> String {
        "https://api.twitter.com/2/users/me".into()
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl OAuth2UserMethods for DiscordUser {
    fn get_user_info_url() -> String {
        "https://discord.com/api/users/@me".into()
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
