# Web Scraping Library

Recursive web scraping, media downloading, and content extraction from websites. Including CAPTCHA solving capability via AI (solves simple captch's).

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
knee_scraper "0.1.8"
reqwest = "0.12.7"
tokio = { version = "1.40.0", features = ["full", "fs"] }
```
## Scrape based on 'keyword search' with => "knee_scraper::rec_scrape;" + new configuration options => "knee_scraper::ScraperConfig;"  

```rust 
use knee_scraper::{ run, ScraperConfig, rec_scrape };
use reqwest::{Client, header};
use std::collections::{HashSet, VecDeque};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Initialize the HTTP client
    let client = Client::new();

    // Set your target_phrase  ((Target phrase not found?  Scraper will discontinue scraping in that direction.))
    let target_phrase = "Hardcore computer-science porn";

    // Set your URL  (( Mine has a 'z' where it shouldn't, whoops i guess i'm clumsy ))
    let url = "httpz://www.happythoughts.com/";
    
    // Initialize the hashset for visited url storage 
    let mut visited = HashSet::new();
    
    // Initialize the ScraperConfig with your default settings
    let config = Some(ScraperConfig::new(
        true,                                   // follow_links: true
        3,                                      // max_depth: 3
        Some("Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X)...".to_string()),  // user_agent
    ));

    // 'Update Logic' -> Should you require these settings change upon condition
    config.set_follow_links(false);
    config.set_max_depth(5);
    config.set_user_agent(Some("UpdatedScraper/2.0(My new updated user agent, brain: CPU Unlimmited learning like a turing machine)...".to_string()));     

    // Print the updated settings...
    println!("Updated follow links: {}", config.follow_links());
    println!("Updated max depth: {}", config.max_depth());
    println!("User agent: {:?}", config.user_agent());


    // Call rec_scrape() and specify config as reference with the as_ref() function.
    rec_scrape(&url, &client, config.as_ref(), &mut visited, target_phrase).await;
    // Without "Config", specify the "None".
    // rec_scrape(&url, &client, None, &mut visited, target_phrase).await;
 
    // If you want a terminal output of completed tasks
    println!("Scraping process completed for {}", url);

    // Optional delay to simulate a more human-like browsing pattern
    sleep(Duration::from_secs(2)).await;
}
```

## Scrape with scrape_js_content() for APIkey or products and/or w/e
```rust  
use knee_scraper::{rec_scrape, scrape_js_content, ScraperConfig};
use reqwest::{Client};
use std::collections::HashSet;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Initialize the HTTP client
    let client = Client::new();

    // Set your target_phrase ((Target phrase not found? Scraper will discontinue scraping in that direction.))
    let target_phrase = "algo";

    // Set your URL ((Don't forget to use a valid URL!))
    let url = "https://www.technology.com/";
    
    // Initialize the hashset for visited URL storage
    let mut visited = HashSet::new();

    // Initialize the ScraperConfig with your default settings
    let mut config = Some(ScraperConfig::new(
        true,                                   // follow_links: true
        3,                                      // max_depth: 3
        Some("Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X)...".to_string()),  // user_agent
    ));

    // 'Update Logic' -> Should you require these settings change upon condition
    config.as_mut().map(|cfg| {
        cfg.set_follow_links(false);
        cfg.set_max_depth(5);
        cfg.set_user_agent(Some(
            "UpdatedScraper/2.0 (My new updated user agent, brain: CPU Unlimited learning like a Turing machine)...".to_string(),
        ));
    });

    // Print the updated settings...
    config.as_mut().map(|cfg| {
        println!("Updated follow links: {}", cfg.follow_links());
        println!("Updated max depth: {}", cfg.max_depth());
        println!("User agent: {:?}", cfg.user_agent());
    });

    // Call rec_scrape() and specify config as reference with the as_ref() function.
    rec_scrape(&url, &client, config.as_ref(), &mut visited, target_phrase).await;

    // Without "Config", specify the "None".
    // rec_scrape(&url, &client, None, &mut visited, target_phrase).await;

    // **Now use `scrape_js_content` to extract JavaScript data**:
    // Define a list of keywords to search for in the JavaScript content.
    let js_keywords = vec!["apiKey", "token", "secret"];

    // Fetch the HTML content for the `scrape_js_content` function.
    let html_content = match client.get(url).send().await {
        Ok(response) => match response.text().await {
            Ok(text) => text,
            Err(_) => {
                eprintln!("Failed to get HTML content from the response");
                return;
            }
        },
        Err(_) => {
            eprintln!("Failed to send request");
            return;
        }
    };

// Call scrape_js_content with the correct arguments
scrape_js_content(&html_content, &url, &client, &js_keywords).await;
    // If you want a terminal output of completed tasks
    println!("Scraping process completed for {}", url);

    // Optional delay to simulate a more human-like browsing pattern
    sleep(Duration::from_secs(2)).await;
}
```

## run() Example - with vector of urls to start from 
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
        "https://example2.com",
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

# With Robots txt & fetch cookies

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
```
# Recursively scrape the content of a website while handling CAPTCHA

```rust
use std::collections::HashSet;
use reqwest::Client;

#[tokio::main]
async fn main() {
    let client = Client::new();
    let mut visited = HashSet::new();
    let target_phrase = "example phrase to find";

    // Starting URL to scrape
    let url = "https://example.com";

    // Perform recursive scraping with CAPTCHA handling
    rec_ai_scrape(url, &client, None, &mut visited, target_phrase).await;
}
```

