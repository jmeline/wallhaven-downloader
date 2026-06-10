mod models;

use std::env::current_dir;
use std::sync::Arc;
use reqwest::Client;
use std::time::Duration;
use anyhow::anyhow;
use models::wallpaper_data::{WallpaperData, WallpaperSearchResults, Data};
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

    pub async fn get_wallpaper_search_results(&self, page: u32) -> Result<WallpaperSearchResults, reqwest::Error> {
        let page = if page > 0 { page } else { 1 };
        let url = format!("{}/search?sorting=toplist&topRange=1y&resolutions=2560x1440&page={page}", self.base_url);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        let wallpaper_search_results = response.json::<WallpaperSearchResults>().await?;
        Ok(wallpaper_search_results)
    }

    pub async fn download_wallpaper(&self, id: &str, path: &str) -> Result<(), anyhow::Error> {
        let response = self.client.get(path).send().await?;
        let bytes = response.bytes().await?;
        // Here you would save the bytes to a file, but for this example, we'll just print the size
        if bytes.is_empty() {
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
        std::fs::write(&file_path, &bytes)?;

        println!("Downloaded wallpaper with size: {} bytes to '{:?}'", bytes.len(), file_path);

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let client = Arc::new(ApiClient {
        client: Client::new(),
        base_url: "https://wallhaven.cc/api/v1".to_owned()
    });

    // let wallpaper = client.get_wallpaper_by_id("k82d26").await.unwrap();
    // client.download_wallpaper(&wallpaper.data.id, &wallpaper.data.path).await.unwrap();
    // println!("Hello, world!");
    // println!("{:#?}", wallpaper);

    // let wallpapers = client.get_wallpaper_search_results(1).await.unwrap();
    // let links: Vec<String> = wallpapers.data.iter().map(|w|w.path.clone()).collect();
    // println!("Found {} wallpapers", links.len());
    // println!("First {links:#?} wallpaper");

    let mass_wallpapers = mass_search_wallpapers(&client, 10).await.unwrap();
    mass_download_wallpapers(&client, &mass_wallpapers).await.unwrap();

}

async fn mass_search_wallpapers(client: &Arc<ApiClient>, range: u32) -> Result<Vec<Data>, anyhow::Error> {
    let concurrency_limit = 8;
    let semaphore = Arc::new(Semaphore::new(concurrency_limit));
    let mut tasks = JoinSet::new();

    for i in 1..=range {
        let client = Arc::clone(client);
        let semaphore = Arc::clone(&semaphore);

        tasks.spawn(async move {
            let _permit = semaphore.acquire_owned().await
                .map_err(|_| anyhow!("Search limiter was closed"))?;

            let result = client.get_wallpaper_search_results(i).await?;
            println!("Found {} wallpapers on page {i}", result.data.len());
            Ok::<_, anyhow::Error>(result)
        });
    }

    let mut results = Vec::new();

    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok(result)) => {
                for item in &result.data {
                    println!("{:?}", item.path);
                }
                results.extend(result.data);
            }
            Ok(Err(err)) => eprintln!("Search failed: {err}"),
            Err(join_err) => eprintln!("Task failed: {join_err}"),
        }
    }

    Ok(results)
}

async fn mass_download_wallpapers(client: &Arc<ApiClient>, wallpapers: &[Data]) -> Result<(), anyhow::Error> {
    let concurrency_limit = 20;
    let semaphore = Arc::new(Semaphore::new(concurrency_limit));
    let mut downloads = JoinSet::new();

    for data in wallpapers {
        let client = Arc::clone(client);
        let semaphore = Arc::clone(&semaphore);
        let id = data.id.clone();
        let path = data.path.clone();

        downloads.spawn(async move {
            let _permit = semaphore.acquire_owned().await
                .map_err(|_| anyhow!("Download limiter was closed"))?;

            println!("Grabbing {}", path);
            client.download_wallpaper(&id, &path).await?;
            println!("Done grabbing {}", path);
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

    Ok(())
}
