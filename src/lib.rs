use reqwest::{header::HeaderName, Url};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct Herokuru {
    client: reqwest::Client,
    pub base_url: Url,
}

impl Default for Herokuru {
    fn default() -> Self {
        Self {
            base_url: Url::parse("https://api.heroku.com/").unwrap(),
            client: reqwest::ClientBuilder::new()
                .user_agent("Reqwest/herokuru version 0.1.0")
                .build()
                .unwrap(),
        }
    }
}

#[derive(Default)]
pub struct HerokuruBuilder {
    token: String,
    base_url: Option<Url>,
}

impl HerokuruBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn base_url(mut self, base_url: impl Into<Option<Url>>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = token.into();
        self
    }

    pub fn build(self) -> Option<Herokuru> {
        let mut hmap = reqwest::header::HeaderMap::new();
        hmap.append(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", self.token).parse().unwrap(),
        );

        let client = reqwest::Client::builder()
            .user_agent("Reqwest/herokuru version 0.1.0")
            .default_headers(hmap)
            .build()
            .unwrap();

        Some(Herokuru {
            client,
            base_url: self
                .base_url
                .unwrap_or_else(|| Url::parse("https://api.heroku.com/").unwrap()),
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
