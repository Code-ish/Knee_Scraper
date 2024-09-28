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
knee_scraper = "0.1.3"
reqwest = "0.12.7"
tokio = { version = "1.40.0", features = ["full", "fs"] }
```
## All inclusive run() function that calls all available features of the Crate 

```rust 
use knee_scraper::run;
use reqwest::Client;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Initialize the HTTP client
    let client = Client::new();

    // Define the URL to scrape
    let url = "https://example.com";

    // Call the run function from knee_scraper to execute the scraping workflow
    println!("Starting the scraping process for {}", url);
    run(url, &client).await;
    println!("Scraping process completed for {}", url);

    // Optional delay to simulate a more human-like browsing pattern
    sleep(Duration::from_secs(2)).await;
}
```
## run() Example2 - with vector of urls to start from 
```rust
use knee_scraper::run;
use reqwest::Client;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Initialize the HTTP client
    let client = Client::new();

    // Define a vector of URLs to scrape
    let urls = vec![
        "https://example.com",
        "https://exampl3e.com",
    ];

    // Loop over each URL and call the `run` function
    for &url in &urls {
        println!("Starting the scraping process for {}", url);
        run(url, &client).await;
        println!("Scraping process completed for {}", url);

        // Optional delay to simulate human-like behavior between scrapes
        sleep(Duration::from_secs(2)).await;
    }
}

```

## Basic Recursive Scraping Examples

```rust
use knee_scraper::recursive_scrape;
use std::collections::HashSet;
use reqwest::Client;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let client = Client::new(); // Initializing the HTTP client
    let mut visited = HashSet::new(); // To track visited URLs

    let base_url = "https://example.com";
    
    //
    // Start recursive scraping from the given URL
    //  Recursive scrape is a hand selected set of functions available
    //   'knee-scraper::run;'  will be the easiest all inclusive  
    recursive_scrape(base_url, &client, &mut visited).await;
    
    // Scrape2 utilizing the 'async fn extract_links()' 
    recursive_scrape2(base_url, &client, &mut visited).await;

}

async fn recursive_scrape2(url: &str, client: &Client, visited: &mut HashSet<String>) {
    if visited.contains(url) {
        return; // If the URL was already visited, skip it
    }

    visited.insert(url.to_string()); // Mark the URL as visited

    // Fetch the HTML content from the current URL
    let response = client.get(url).send().await.unwrap();

    if response.status().is_success() {
        let html = response.text().await.unwrap();

        // Extract links from the HTML content
        let links = knee_scraper::extract_links(&html, url);

        println!("Scraped {} - Found {} links", url, links.len());

        // Recursively scrape each extracted link
        for link in links {
            // Avoid re-scraping the same URLs
            if !visited.contains(&link) {
                recursive_scrape(&link, client, visited).await;
                sleep(Duration::from_millis(500)).await; // Add a delay between requests to avoid overwhelming the server
            }
        }
    }
}

```

## Example with Robots.txt, Open Directories, and Cookies

```toml
[dependencies]
knee_scraper = "0.1.3"
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
use tokio::time::{sleep, Duration};


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


```toml
[dependencies]
knee_scraper = "0.1.3"
futures = "0.3.30"
rand = "0.8.5"
regex = "1.10.6"
reqwest = "0.12.7"
scraper = "0.20.0"
tokio = { version = "1.40.0", features = ["full", "fs"] }
url = "2.5.2"

```

```rust


```