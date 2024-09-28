# Web Scraping Library

This Rust library enables recursive web scraping, media downloading, and content extraction from websites. With robust error handling and media support, the library is designed for flexible and scalable use in various web scraping scenarios.

## Features

- **Recursive Scraping**: Start scraping from any URL and recursively follow links.
- **Media Downloading**: Download images, videos, and other media assets.
- **Content Extraction**: Extract text, meta tags, forms, and JavaScript contents from web pages.
- **Error Logging**: Logs errors to a file for later analysis.
- **Random Delays**: Mimics human behavior by adding random delays between requests.

## Installation

To install the library, add the following to your `Cargo.toml`:

```toml
[dependencies]
knee_scraper = "0.1.0"
```

## Basic Recursive Scraping Example

```rust
use knee_scraper::{Client, recursive_scrape};
use std::collections::HashSet;

#[tokio::main]
async fn main() {
    let client = Client::new(); // Initializing the HTTP client
    let mut visited = HashSet::new(); // To track visited URLs

    // Start recursive scraping from the given URL
    recursive_scrape("https://example.com", &client, &mut visited).await;
}

let html = "<a href='/about'>About Us</a>";
let base_url = "https://example.com";

// Extract links from the HTML content
let links = knee_scraper::extract_links(html, base_url);
```

## Advanced Example with Robots.txt, Open Directories, and Cookies

```toml
[dependencies]
knee_scraper = "0.1.0"
futures = "0.3.30"
rand = "0.8.5"
regex = "1.10.6"
reqwest = "0.12.7"
scraper = "0.20.0"
tokio = { version = "1.40.0", features = ["full", "fs"] }
url = "2.5.2"

```

```rust
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
