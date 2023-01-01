use argon2::{
    password_hash::{
        rand_core::OsRng, Error, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};

pub mod oauth2;

pub fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    Ok(argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, Error> {
    let hash = PasswordHash::new(hash)?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &hash)
        .is_ok())
}
