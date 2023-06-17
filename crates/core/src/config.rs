use std::{collections::HashMap, env, fmt};

use crate::{entities::System, error::Error};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ConfigKey {
    CurrentVersion,
    PreviousVersion,
    CertsPath,
    UseTls,
    ClientUrl,
    ServerUrl,
    SecretKey,
    SmtpHost,
    SmtpPort,
    SmtpUsername,
    SmtpPassword,
    SmtpSenderName,
    SmtpSenderEmail,
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
            ConfigKey::CurrentVersion => "CURRENT_VERSION",
            ConfigKey::PreviousVersion => "PREVIOUS_VERSION",
            ConfigKey::CertsPath => "CERTS_PATH",
            ConfigKey::UseTls => "USE_TLS",
            ConfigKey::ClientUrl => "CLIENT_URL",
            ConfigKey::ServerUrl => "SERVER_URL",
            ConfigKey::SecretKey => "SECRET_KEY",
            ConfigKey::CockroachUrl => "COCKROACH_URL",
            ConfigKey::DragonflyUrl => "DRAGONFLY_URL",
            ConfigKey::SmtpHost => "SMTP_HOST",
            ConfigKey::SmtpPort => "SMTP_PORT",
            ConfigKey::SmtpUsername => "SMTP_USERNAME",
            ConfigKey::SmtpPassword => "SMTP_PASSWORD",
            ConfigKey::SmtpSenderName => "SMTP_SENDER_NAME",
            ConfigKey::SmtpSenderEmail => "SMTP_SENDER_EMAIL",
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

#[derive(Clone, Debug)]
struct Entry<'a> {
    keys: Vec<ConfigKey>,
    system: bool,
    required: bool,
    default: Option<&'a str>,
}

#[derive(Clone, Default, Debug)]
pub struct Config(HashMap<ConfigKey, Option<String>>, Option<System>);

impl Config {
    fn options() -> Vec<Entry<'static>> {
        vec![
            Entry {
                keys: vec![ConfigKey::CertsPath],
                system: false,
                required: false,
                default: Some("../../certs"),
            },
            Entry {
                keys: vec![ConfigKey::UseTls],
                system: false,
                required: false,
                default: Some("true"),
            },
            Entry {
                keys: vec![ConfigKey::ClientUrl],
                system: false,
                required: false,
                default: Some("https://localhost:3000"),
            },
            Entry {
                keys: vec![ConfigKey::ServerUrl],
                system: false,
                required: false,
                default: Some("localhost:8000"),
            },
            Entry {
                keys: vec![ConfigKey::SecretKey],
                system: false,
                required: false,
                default: Some("l19OOJaOvpal21ofSlsxYyNVQN2EeTY6gEuq6p_hobH_QmJb3dPELmoGKFstBpCI"),
            },
            Entry {
                keys: vec![ConfigKey::CockroachUrl, ConfigKey::DragonflyUrl],
                system: false,
                required: true,
                default: None,
            },
            Entry {
                keys: vec![
                    ConfigKey::CurrentVersion,
                    ConfigKey::PreviousVersion,
                    ConfigKey::SmtpHost,
                    ConfigKey::SmtpPort,
                    ConfigKey::SmtpUsername,
                    ConfigKey::SmtpPassword,
                    ConfigKey::SmtpSenderName,
                    ConfigKey::SmtpSenderEmail,
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
                system: true,
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

    pub fn attach_system(&mut self, system: System) {
        self.1 = Some(system.clone());

        Config::options()
            .iter()
            .filter(|entry| entry.system)
            .for_each(|entry| {
                entry.keys.iter().for_each(|key| {
                    let value = match key {
                        ConfigKey::CurrentVersion => Some(system.current_version.clone()),
                        ConfigKey::PreviousVersion => system.previous_version.clone(),
                        ConfigKey::SmtpHost => system.smtp_config.clone().map(|c| c.host),
                        ConfigKey::SmtpPort => {
                            system.smtp_config.clone().map(|c| c.port.to_string())
                        }
                        ConfigKey::SmtpUsername => system.smtp_config.clone().map(|c| c.username),
                        ConfigKey::SmtpPassword => system.smtp_config.clone().map(|c| c.password),
                        ConfigKey::SmtpSenderName => {
                            system.smtp_config.clone().map(|c| c.sender_name)
                        }
                        ConfigKey::SmtpSenderEmail => {
                            system.smtp_config.clone().map(|c| c.sender_email)
                        }

                        ConfigKey::GoogleClientId => {
                            system.google_config.clone().map(|c| c.client_id)
                        }
                        ConfigKey::GoogleClientSecret => {
                            system.google_config.clone().map(|c| c.client_secret)
                        }
                        ConfigKey::FacebookClientId => {
                            system.facebook_config.clone().map(|c| c.client_id)
                        }
                        ConfigKey::FacebookClientSecret => {
                            system.facebook_config.clone().map(|c| c.client_secret)
                        }
                        ConfigKey::GitHubClientId => {
                            system.github_config.clone().map(|c| c.client_id)
                        }
                        ConfigKey::GitHubClientSecret => {
                            system.github_config.clone().map(|c| c.client_secret)
                        }
                        ConfigKey::TwitterClientId => {
                            system.twitter_config.clone().map(|c| c.client_id)
                        }
                        ConfigKey::TwitterClientSecret => {
                            system.twitter_config.clone().map(|c| c.client_secret)
                        }
                        ConfigKey::DiscordClientId => {
                            system.discord_config.clone().map(|c| c.client_id)
                        }
                        ConfigKey::DiscordClientSecret => {
                            system.discord_config.clone().map(|c| c.client_secret)
                        }

                        _ => None,
                    };

                    self.0.insert(key.clone(), value);
                });
            });
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
