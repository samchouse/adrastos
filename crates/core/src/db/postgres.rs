use core::fmt;
use std::{fs::File, io::BufReader};

use deadpool_postgres::{
    tokio_postgres::{Config, NoTls},
    Manager, ManagerConfig, Pool, RecyclingMethod,
};
use rustls::{Certificate, ClientConfig, RootCertStore};
use rustls_pemfile::certs;
use tokio_postgres::config::SslMode;
use tokio_postgres_rustls::MakeRustlsConnect;

use crate::config;

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
    let pg_config = config.postgres_url.parse::<Config>().unwrap();
    let manager_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };

    let mgr = match pg_config.get_ssl_mode() {
        SslMode::Disable => Manager::from_config(pg_config, NoTls, manager_config),
        _ => {
            let Some(certs_path) = &config.certs_path else {
                panic!("Certs path not set")
            };

            let ca_cert = &mut BufReader::new(
                File::open(format!("{}/cockroach.crt", certs_path)).unwrap(),
            );
            let ca_cert = Certificate(certs(ca_cert).unwrap()[0].clone());

            let mut root_store = RootCertStore::empty();
            root_store.add(&ca_cert).unwrap();

            let config = ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(root_store)
                .with_no_client_auth();

            Manager::from_config(pg_config, MakeRustlsConnect::new(config), manager_config)
        }
    };

    Pool::builder(mgr).max_size(16).build().unwrap()
}
