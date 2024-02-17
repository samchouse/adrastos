use webauthn_rs::{prelude::Url, Webauthn, WebauthnBuilder};

pub fn build_webauthn(origin: &str) -> Webauthn {
    WebauthnBuilder::new(
        Url::parse(origin).unwrap().host_str().unwrap(),
        &Url::parse(origin).unwrap(),
    )
    .unwrap()
    .rp_name("Adrastos")
    .build()
    .unwrap()
}
