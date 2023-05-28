use core::fmt;

use deadpool_postgres::{
    tokio_postgres::{Config, NoTls},
    Manager, ManagerConfig, Pool, RecyclingMethod,
};
use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::MakeTlsConnector;
use tokio_postgres::config::SslMode;

use crate::config::{self, ConfigKey};

pub enum Error {
    UniqueKeyViolation,
    NonExistentTable,
}

impl TryFrom<&str> for Error {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            _ if value == Error::UniqueKeyViolation.to_string() => Ok(Error::UniqueKeyViolation),
            _ if value == Error::NonExistentTable.to_string() => Ok(Error::NonExistentTable),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Error::UniqueKeyViolation => "NewUniquenessConstraintViolationError",
            Error::NonExistentTable => "NewUndefinedRelationError",
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
    let manager_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };

    let mgr = match pg_config.get_ssl_mode() {
        SslMode::Disable => Manager::from_config(pg_config, NoTls, manager_config),
        _ => {
            let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
            builder
                .set_ca_file(format!(
                    "{}/cockroach.crt",
                    config.get(ConfigKey::CertsPath).unwrap()
                ))
                .unwrap();
            let connector = MakeTlsConnector::new(builder.build());

            Manager::from_config(pg_config, connector, manager_config)
        }
    };

    Pool::builder(mgr).max_size(16).build().unwrap()
}
