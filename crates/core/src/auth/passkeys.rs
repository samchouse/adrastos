use axum::http::HeaderValue;
use webauthn_rs::{prelude::Url, Webauthn, WebauthnBuilder};

use crate::{config::Config, entities::Project};

pub async fn build_webauthn(
    host: Option<&HeaderValue>,
    project: &Option<Project>,
    config: &Config,
) -> Webauthn {
    let client_url = config.client_url.as_str();
    let origin = if let Ok(url) = Url::parse(client_url)
        && url.host().is_some()
    {
        client_url
    } else {
        host.unwrap().to_str().unwrap()
    };

    WebauthnBuilder::new(
        Url::parse(origin).unwrap().host_str().unwrap(),
        &Url::parse(origin).unwrap(),
    )
    .unwrap()
    .rp_name(
        project
            .as_ref()
            .map(|p| p.name.as_str())
            .unwrap_or("Adrastos"),
    )
    .build()
    .unwrap()
}
