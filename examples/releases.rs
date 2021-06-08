use herokuru::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let heroku = Herokuru::builder()
        .token(std::env::var("HEROKU_TOKEN")?)
        .build()
        .unwrap();
    let mut page = Some(Page::first_releases());
    while let Some(res) = heroku
        .releases(std::env::var("APP_NAME")?)
        .list(page)
        .await?
    {
        for release in res.releases.into_iter() {
            println!("release: {}", release.version);
        }
        page = res.next;
    }
    println!("OK");
    Ok(())
}
