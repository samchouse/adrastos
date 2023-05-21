use std::fmt;

pub enum SessionKey {
    UserId,
    CsrfToken,
}

impl fmt::Display for SessionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            SessionKey::UserId => "user_id",
            SessionKey::CsrfToken => "csrf_token",
        };

        write!(f, "{name}")
    }
}
