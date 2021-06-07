use chrono::{DateTime, Utc};
use reqwest::{header::HeaderValue, Url};
use serde::{Deserialize, Serialize};

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
        hmap.append(
            reqwest::header::ACCEPT,
            "application/vnd.heroku+json; version=3".parse().unwrap(),
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

#[derive(Debug, Clone, Default)]
pub struct Page {
    pub key: String,
    pub order: String,
    pub per_page: u32,

    pub range_format: String,
}

impl Page {
    pub fn first_releases() -> Self {
        Self {
            key: "version".to_string(),
            order: "desc".to_string(),
            per_page: 1000u32,
            ..Default::default()
        }
        .gen_range_format()
    }

    fn gen_range_format(mut self) -> Self {
        self.range_format = format!(
            "{} ; order={},max={}",
            &self.key, &self.order, &self.per_page
        );
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Release {
    pub id: String, // is UUID
    pub addon_plan_names: Vec<String>,
    pub app: App,
    pub created_at: DateTime<Utc>,
    pub description: String,
    pub status: String,
    pub slug: Slug,
    pub updated_at: DateTime<Utc>,
    pub user: User,
    pub version: i32,
    pub current: bool,
    pub output_stream_url: Url,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct App {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Slug {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct Herokuru {
    client: reqwest::Client,
    pub base_url: Url,
}

#[derive(Debug, Clone)]
pub struct ReleaseResponse {
    releases: Vec<Release>,
    next: Option<Page>,
}

impl Herokuru {
    pub async fn releases(
        &self,
        app_name: impl Into<String>,
        page: Option<Page>,
    ) -> Result<Option<ReleaseResponse>, Box<dyn std::error::Error>> {
        match page {
            None => Ok(None),
            Some(page) => {
                let path = format!("apps/{}/releases", app_name.into());
                let url = self.base_url.join(&path)?;

                let res = self
                    .client
                    .get(url)
                    .header("Range", page.range_format.parse::<HeaderValue>().unwrap())
                    .send()
                    .await?;
                let headers = res.headers();

                let next = headers.get("next-range").map(|range_format| Page {
                    range_format: range_format.to_str().unwrap().into(),
                    ..Default::default()
                });
                let json: serde_json::Value = res.json().await?;
                let releases: Vec<Release> = serde_json::from_value(json)?;

                Ok(ReleaseResponse { releases, next }.into())
            }
        }
    }
}
