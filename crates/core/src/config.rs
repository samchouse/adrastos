use std::{collections::HashMap, env, fmt};

use crate::error::Error;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ConfigKey {
    CertsPath,
    ClientUrl,
    ServerUrl,
    SecretKey,
    SmtpHost,
    SmtpPort,
    SmtpUsername,
    SmtpPassword,
    CockroachUrl,
    DragonflyUrl,
    GoogleClientId,
    GoogleClientSecret,
    FacebookClientId,
    FacebookClientSecret,
    GitHubClientId,
    GitHubClientSecret,
    TwitterClientId,
    TwitterClientSecret,
    DiscordClientId,
    DiscordClientSecret,
}

impl fmt::Display for ConfigKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ConfigKey::CertsPath => "CERTS_PATH",
            ConfigKey::ClientUrl => "CLIENT_URL",
            ConfigKey::ServerUrl => "SERVER_URL",
            ConfigKey::SecretKey => "SECRET_KEY",
            ConfigKey::SmtpHost => "SMTP_HOST",
            ConfigKey::SmtpPort => "SMTP_PORT",
            ConfigKey::SmtpUsername => "SMTP_USERNAME",
            ConfigKey::SmtpPassword => "SMTP_PASSWORD",
            ConfigKey::CockroachUrl => "COCKROACH_URL",
            ConfigKey::DragonflyUrl => "DRAGONFLY_URL",
            ConfigKey::GoogleClientId => "GOOGLE_CLIENT_ID",
            ConfigKey::GoogleClientSecret => "GOOGLE_CLIENT_SECRET",
            ConfigKey::FacebookClientId => "FACEBOOK_CLIENT_ID",
            ConfigKey::FacebookClientSecret => "FACEBOOK_CLIENT_SECRET",
            ConfigKey::GitHubClientId => "GITHUB_CLIENT_ID",
            ConfigKey::GitHubClientSecret => "GITHUB_CLIENT_SECRET",
            ConfigKey::TwitterClientId => "TWITTER_CLIENT_ID",
            ConfigKey::TwitterClientSecret => "TWITTER_CLIENT_SECRET",
            ConfigKey::DiscordClientId => "DISCORD_CLIENT_ID",
            ConfigKey::DiscordClientSecret => "DISCORD_CLIENT_SECRET",
        };

        write!(f, "{name}")
    }
}

struct Entry<'a> {
    keys: Vec<ConfigKey>,
    required: bool,
    default: Option<&'a str>,
}

#[derive(Clone, Default)]
pub struct Config(HashMap<ConfigKey, Option<String>>);

impl Config {
    fn options() -> Vec<Entry<'static>> {
        vec![
            Entry {
                keys: vec![ConfigKey::CertsPath],
                required: false,
                default: Some("../../certs"),
            },
            Entry {
                keys: vec![ConfigKey::ClientUrl],
                required: false,
                default: Some("https://127.0.0.1:3000"),
            },
            Entry {
                keys: vec![ConfigKey::ServerUrl],
                required: false,
                default: Some("https://127.0.0.1:8000"),
            },
            Entry {
                keys: vec![ConfigKey::SecretKey],
                required: false,
                default: Some("l19OOJaOvpal21ofSlsxYyNVQN2EeTY6gEuq6p_hobH_QmJb3dPELmoGKFstBpCI"),
            },
            Entry {
                keys: vec![
                    ConfigKey::CockroachUrl,
                    ConfigKey::DragonflyUrl,
                    ConfigKey::SmtpHost,
                    ConfigKey::SmtpPort,
                    ConfigKey::SmtpUsername,
                    ConfigKey::SmtpPassword,
                ],
                required: true,
                default: None,
            },
            Entry {
                keys: vec![
                    ConfigKey::GoogleClientId,
                    ConfigKey::GoogleClientSecret,
                    ConfigKey::FacebookClientId,
                    ConfigKey::FacebookClientSecret,
                    ConfigKey::GitHubClientId,
                    ConfigKey::GitHubClientSecret,
                    ConfigKey::TwitterClientId,
                    ConfigKey::TwitterClientSecret,
                    ConfigKey::DiscordClientId,
                    ConfigKey::DiscordClientSecret,
                ],
                required: false,
                default: None,
            },
        ]
    }

    pub fn new() -> Result<Self, Vec<String>> {
        let mut config = Config::default();
        let mut missing_keys = vec![];

        Config::options().iter().for_each(|entry| {
            entry.keys.iter().for_each(|key| {
                let value = match env::var(key.to_string()) {
                    Ok(value) => Some(value),
                    Err(error) => match error {
                        env::VarError::NotPresent => {
                            if let Some(default) = &entry.default {
                                Some(default.to_string())
                            } else {
                                if entry.required {
                                    missing_keys.push(key.to_string())
                                }

                                None
                            }
                        }
                        err => panic!("{}", err),
                    },
                };

                config.0.insert(key.clone(), value);
            });
        });

        if !missing_keys.is_empty() {
            return Err(missing_keys);
        }

        Ok(config)
    }

    pub fn get(&self, key: ConfigKey) -> Result<String, Error> {
        self.0
            .get(&key)
            .ok_or(Error::InternalServerError(
                "Unable to get config value".into(),
            ))?
            .to_owned()
            .ok_or(Error::InternalServerError("No value for config key".into()))
    }
}
