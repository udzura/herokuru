extern crate herokuru;
extern crate tokio;

use chrono::{DateTime, Utc};
use csv::WriterBuilder;
use herokuru::*;
use log::*;
use reqwest::Url;
use serde::*;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Command {
    /// Specify a resource to download. Only `releases' is supported for now
    #[structopt(name = "resource")]
    resource: String,
    /// Specify app's name on Heroku
    #[structopt(long = "app-name", short = "A")]
    app_name: String,
}

#[derive(Deserialize, Debug)]
struct Env {
    heroku_token: String,
    heroku_base_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[non_exhaustive]
pub struct ReleaseCsv {
    pub id: String, // is UUID
    pub version: i32,
    pub addon_plan_names: String,
    pub app: String,
    pub status: String,
    pub slug: Option<String>,
    pub user_email: String,
    pub current: bool,
    pub description: String,
    pub output_stream_url: Option<Url>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Release> for ReleaseCsv {
    fn from(from: Release) -> Self {
        Self {
            id: from.id,
            addon_plan_names: serde_json::to_string(&from.addon_plan_names).unwrap(),
            app: from.app.name,
            created_at: from.created_at,
            description: from.description,
            status: from.status,
            slug: from.slug.map(|s| s.id),
            updated_at: from.updated_at,
            user_email: from.user.email,
            version: from.version,
            current: from.current,
            output_stream_url: from.output_stream_url,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let config: Env = envy::from_env().expect("while reading from environment");

    let args = Command::from_args();

    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(std::io::stdout());

    match args.resource.as_str() {
        "releases" => {
            let heroku = Herokuru::builder()
                .token(config.heroku_token)
                .build()
                .unwrap();
            let mut page = Some(Page::first_releases());
            while let Some(res) = heroku.releases(&args.app_name).list(page).await? {
                for release in res.releases.into_iter() {
                    let row: ReleaseCsv = release.into();
                    wtr.serialize(&row)?;
                }
                page = res.next;
            }
        }
        _ => {
            error!("Unsupported: {}", args.resource);
            panic!("panic")
        }
    }

    Ok(())
}
