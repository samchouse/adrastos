use core::fmt;

use deadpool_postgres::{
    tokio_postgres::{Config, NoTls},
    Manager, ManagerConfig, Pool, RecyclingMethod,
};

use crate::config::{self, ConfigKey};

pub enum Error {
    UniqueKeyViolation,
}

impl TryFrom<&str> for Error {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            _ if value == Error::UniqueKeyViolation.to_string() => Ok(Error::UniqueKeyViolation),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Error::UniqueKeyViolation => "NewUniquenessConstraintViolationError",
        };

        write!(f, "{name}")
    }
}

pub fn create_pool(config: &config::Config) -> Pool {
    let pg_config = config
        .get(ConfigKey::CockroachUrl)
        .unwrap()
        .parse::<Config>()
        .unwrap();
    let mgr = Manager::from_config(
        pg_config,
        NoTls,
        ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        },
    );

    Pool::builder(mgr).max_size(16).build().unwrap()
}
