use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub url: Option<String>,
    pub cockroach_url: Option<String>,
    pub dragonfly_url: Option<String>,
}

struct Entry<'a> {
    key: &'a str,
    required: bool,
    default: Option<&'a str>,
}

const CONFIG_ENTRIES: [Entry; 3] = [
    Entry {
        key: "URL",
        required: false,
        default: Some("127.0.0.1:8000"),
    },
    Entry {
        key: "COCKROACH_URL",
        required: true,
        default: None,
    },
    Entry {
        key: "DRAGONFLY_URL",
        required: true,
        default: None,
    },
];

impl Config {
    pub fn new() -> Result<Self, Vec<&'static str>> {
        let mut config = Self {
            url: None,
            cockroach_url: None,
            dragonfly_url: None,
        };

        let mut missing_keys = vec![];

        CONFIG_ENTRIES.iter().for_each(|entry| {
            let value = match env::var(entry.key.clone()) {
                Ok(value) => Some(value),
                Err(error) => match error {
                    env::VarError::NotPresent => {
                        if let Some(default) = &entry.default {
                            Some(default.to_string())
                        } else {
                            if entry.required {
                                missing_keys.push(entry.key.clone())
                            }

                            None
                        }
                    }
                    err => panic!("{}", err),
                },
            };

            match entry.key {
                "URL" => config.url = value,
                "COCKROACH_URL" => config.cockroach_url = value,
                "DRAGONFLY_URL" => config.dragonfly_url = value,
                _ => {}
            }
        });

        if missing_keys.len() != 0 {
            return Err(missing_keys);
        }

        Ok(config)
    }
}
