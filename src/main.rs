mod models;

use std::env::current_dir;
use std::sync::Arc;
use reqwest::Client;
use std::time::Duration;
use anyhow::anyhow;
use models::wallpaper_data::WallpaperData;
use crate::models::wallpaper_data::WallpaperSearchResults;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: String) -> Result<Self, reqwest::Error> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .build()?;
        Ok(Self { client, base_url })
    }
}

impl ApiClient {
    pub async fn get_wallpaper_by_id(&self, id: &str) -> Result<WallpaperData, reqwest::Error> {
        let url = format!("{}/w/{}", self.base_url, id);
        let response = self.client.get(&url).send().await?;
        let wallpaper_data = response.json::<WallpaperData>().await?;
        Ok(wallpaper_data)
    }

    pub async fn get_wallpaper_search_results(&self) -> Result<WallpaperSearchResults, reqwest::Error> {
        let url = format!("{}/search?sorting=toplist&topRange=1y&resolutions=2560x1440", self.base_url);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        let wallpaper_search_results = response.json::<WallpaperSearchResults>().await?;
        Ok(wallpaper_search_results)
    }

    pub async fn download_wallpaper(&self, id: &str, path: &str) -> Result<(), anyhow::Error> {
        let response = self.client.get(path).send().await?;
        let bytes = response.bytes().await?;
        // Here you would save the bytes to a file, but for this example, we'll just print the size
        println!("Downloaded wallpaper with size: {} bytes", bytes.len());
        if bytes.len() == 0 {
            println!("Failed to download wallpaper: empty response");
            return Err(anyhow!("Failed to download wallpaper"));
        }

        // Save wallpaper files under ./wallpapers, creating the directory if needed.
        let wallpapers_dir = current_dir()?.join("wallpapers");
        std::fs::create_dir_all(&wallpapers_dir)?;
        let extension = path
            .rsplit('.')
            .next()
            .and_then(|ext| ext.split('?').next())
            .filter(|ext| !ext.is_empty())
            .unwrap_or("jpg");
        let file_path = wallpapers_dir.join(format!("{}.{}", id, extension));
        std::fs::write(file_path, bytes)?;

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let client = Arc::new(ApiClient {
        client: Client::new(),
        base_url: "https://wallhaven.cc/api/v1".to_owned()
    });

    let wallpaper = client.get_wallpaper_by_id("k82d26").await.unwrap();
    client.download_wallpaper(&wallpaper.data.id, &wallpaper.data.path).await.unwrap();
    println!("Hello, world!");
    println!("{:#?}", wallpaper);

    let wallpapers = client.get_wallpaper_search_results().await.unwrap();
    let links: Vec<String> = wallpapers.data.iter().map(|w|w.path.clone()).collect();
    println!("Found {} wallpapers", links.len());
    println!("First {links:#?} wallpaper");

    let concurrency_limit = 8;
    let semaphore = Arc::new(Semaphore::new(concurrency_limit));
    let mut downloads = JoinSet::new();

    for data in wallpapers.data {
        let client = Arc::clone(&client);
        let semaphore = Arc::clone(&semaphore);

        downloads.spawn(async move {
            let permit = semaphore.acquire_owned().await;
            if permit.is_err() {
                return Err(anyhow!("Download limiter was closed"));
            }

            println!("Grabbing {}", data.path);
            client.download_wallpaper(&data.id, &data.path).await?;
            println!("Done grabbing {}", data.path);
            Ok::<(), anyhow::Error>(())
        });
    }

    while let Some(result) = downloads.join_next().await {
        match result {
            Ok(Ok(())) => {}
            Ok(Err(err)) => eprintln!("Download failed: {err}"),
            Err(join_err) => eprintln!("Task failed: {join_err}"),
        }
    }
}
