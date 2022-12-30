use nanoid::{alphabet, nanoid};
use regex::Regex;

#[derive(Debug)]
pub struct Id;

impl Id {
    pub fn new() -> String {
        let regex = Regex::new(r"[\da-zA-Z]").unwrap();
        let alphabet = alphabet::SAFE
            .into_iter()
            .filter(|char| regex.is_match(&char.to_string()))
            .collect::<Vec<char>>();

        nanoid!(20, &alphabet)
    }
}
