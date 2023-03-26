use std::fmt;

use nanoid::{alphabet, nanoid};
use regex::Regex;

#[derive(Debug)]
pub struct Id(String);

impl Id {
    pub fn new() -> Self {
        let regex = Regex::new(r"[\da-zA-Z]").unwrap();
        let alphabet = alphabet::SAFE
            .into_iter()
            .filter(|char| regex.is_match(&char.to_string()))
            .collect::<Vec<char>>();

        Id(nanoid!(20, &alphabet))
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
