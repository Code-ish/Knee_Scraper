use knee_scraper::{recursive_scrape, fetch_robots_txt, check_open_directories, fetch_with_cookies};
use reqwest::Client;
use std::collections::HashSet;
use tokio::time::sleep;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Define the URL to scrape
    let url = "https://example.com"; // Replace this with your target URL

    // Initialize the HTTP client
    let client = Client::new();

    // Initialize a set to track visited URLs
    let mut visited = HashSet::new();

    // Fetch and process robots.txt file
    println!("Fetching robots.txt...");
    fetch_robots_txt(url, &client).await;

    // Check for common open directories
    println!("Checking open directories...");
    check_open_directories(url, &client).await;

    // Fetch page with cookies
    println!("Fetching page with cookies...");
    fetch_with_cookies(url, &client).await;

    // Perform recursive scraping on the URL
    println!("Starting recursive scrape...");
    recursive_scrape(url, &client, &mut visited).await;

    // Adding a delay to simulate human browsing behavior
    println!("Delaying to mimic human behavior...");
    sleep(Duration::from_secs(3)).await;

    println!("Scraping complete.");
}
