use fancy_regex::Regex;

#[derive(Debug)]
pub struct Url {
    pub subdomains: Vec<String>,
    pub domain: String,
    pub tld: String,
}

impl Url {
    pub fn validate_with_patterns(&mut self, patterns: Vec<String>) -> Vec<(bool, String)> {
        patterns
            .iter()
            .map(|pattern| {
                let pattern_url = Url::from(pattern.to_owned());

                let subdomain_diff = if self.subdomains.len() <= pattern_url.subdomains.len() {
                    0
                } else {
                    self.subdomains.len() - pattern_url.subdomains.len()
                };

                let checks = vec![
                    self.subdomains.len() >= pattern_url.subdomains.len(),
                    self.subdomains
                        .drain(0..subdomain_diff)
                        .collect::<Vec<_>>()
                        .iter()
                        .enumerate()
                        .all(|(i, subdomain)| {
                            if let Some(pattern_subdomain) = pattern_url.subdomains.get(i) {
                                if pattern_subdomain == "*" {
                                    return true;
                                }

                                subdomain == pattern_subdomain
                            } else {
                                false
                            }
                        }),
                    self.domain == pattern_url.domain,
                    self.tld == pattern_url.tld,
                ];

                (checks.iter().all(|c| *c), pattern.to_owned())
            })
            .collect()
    }
}

impl From<String> for Url {
    fn from(url: String) -> Self {
        let pre = Regex::new(r"^[\w]+:\/\/").unwrap();
        let post = Regex::new(r"(\/.+|&.+)$").unwrap();

        let url = pre.replace(&url, "").into_owned();
        let url = post.replace(&url, "").into_owned();

        let subdomains = Regex::new(r"[\w\d\*\.]+(?=\.[\w\d]+\.[\w]+$)").unwrap();
        let domain = Regex::new(r"[\w\d]+(?=\.[\w]+$)").unwrap();
        let tld = Regex::new(r"\.[\w]+$").unwrap();

        Url {
            subdomains: match subdomains.find(&url).unwrap() {
                Some(re_match) => re_match.as_str().split('.').map(|s| s.to_owned()).collect(),
                None => vec![],
            },
            domain: domain.find(&url).unwrap().unwrap().as_str().to_owned(),
            tld: tld
                .find(&url)
                .unwrap()
                .unwrap()
                .as_str()
                .to_owned()
                .replace('.', ""),
        }
    }
}
