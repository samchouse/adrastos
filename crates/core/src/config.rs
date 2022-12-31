use std::{collections::HashMap, env};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ConfigKey {
    SecretKey,
    Url,
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

struct Entry<'a> {
    keys: Vec<ConfigKey>,
    required: bool,
    default: Option<&'a str>,
}

#[derive(Clone)]
pub struct Config(HashMap<ConfigKey, Option<String>>);

impl ToString for ConfigKey {
    fn to_string(&self) -> String {
        match self {
            ConfigKey::SecretKey => "SECRET_KEY".to_string(),
            ConfigKey::Url => "URL".to_string(),
            ConfigKey::CockroachUrl => "COCKROACH_URL".to_string(),
            ConfigKey::DragonflyUrl => "DRAGONFLY_URL".to_string(),
            ConfigKey::GoogleClientId => "GOOGLE_CLIENT_ID".to_string(),
            ConfigKey::GoogleClientSecret => "GOOGLE_CLIENT_SECRET".to_string(),
            ConfigKey::FacebookClientId => "FACEBOOK_CLIENT_ID".to_string(),
            ConfigKey::FacebookClientSecret => "FACEBOOK_CLIENT_SECRET".to_string(),
            ConfigKey::GitHubClientId => "GITHUB_CLIENT_ID".to_string(),
            ConfigKey::GitHubClientSecret => "GITHUB_CLIENT_SECRET".to_string(),
            ConfigKey::TwitterClientId => "TWITTER_CLIENT_ID".to_string(),
            ConfigKey::TwitterClientSecret => "TWITTER_CLIENT_SECRET".to_string(),
            ConfigKey::DiscordClientId => "DISCORD_CLIENT_ID".to_string(),
            ConfigKey::DiscordClientSecret => "DISCORD_CLIENT_SECRET".to_string(),
        }
    }
}

impl Config {
    fn options() -> Vec<Entry<'static>> {
        vec![
            Entry {
                keys: vec![ConfigKey::SecretKey],
                required: true,
                default: Some("l19OOJaOvpal21ofSlsxYyNVQN2EeTY6gEuq6p_hobH_QmJb3dPELmoGKFstBpCI"),
            },
            Entry {
                keys: vec![ConfigKey::Url],
                required: false,
                default: Some("127.0.0.1:8000"),
            },
            Entry {
                keys: vec![ConfigKey::CockroachUrl, ConfigKey::DragonflyUrl],
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
                let value = match env::var(key.to_string().clone()) {
                    Ok(value) => Some(value),
                    Err(error) => match error {
                        env::VarError::NotPresent => {
                            if let Some(default) = &entry.default {
                                Some(default.to_string())
                            } else {
                                if entry.required {
                                    missing_keys.push(key.to_string().clone())
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

        if missing_keys.len() != 0 {
            return Err(missing_keys);
        }

        Ok(config)
    }

    pub fn get(&self, key: ConfigKey) -> Option<String> {
        self.0.get(&key)?.to_owned()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self(HashMap::new())
    }
}
