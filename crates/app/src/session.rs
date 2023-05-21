use std::fmt;

pub enum SessionKey {
    UserId,
    CsrfToken,
    LoginUserId,
    MfaRetries,
}

impl fmt::Display for SessionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            SessionKey::UserId => "user_id",
            SessionKey::CsrfToken => "csrf_token",
            SessionKey::LoginUserId => "login_user_id",
            SessionKey::MfaRetries => "mfa_retries",
        };

        write!(f, "{name}")
    }
}
