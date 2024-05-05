use std::fmt;

pub enum SessionKey {
    UserId,
    UserType,
    CsrfToken,
    LoginUserId,
    MfaRetries,
    Redirect,
    PasskeyRegistration,
    PasskeyAuthentication,
}

impl fmt::Display for SessionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            SessionKey::UserId => "user_id",
            SessionKey::UserType => "user_type",
            SessionKey::CsrfToken => "csrf_token",
            SessionKey::LoginUserId => "login_user_id",
            SessionKey::MfaRetries => "mfa_retries",
            SessionKey::Redirect => "redirect",
            SessionKey::PasskeyRegistration => "passkey_registration",
            SessionKey::PasskeyAuthentication => "passkey_authentication",
        };

        write!(f, "{name}")
    }
}
