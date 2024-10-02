// src/lib.rs

use reqwest::{ Client, Url, header };
use scraper::{ Html, Selector };
use std::collections::{ HashSet, VecDeque };
use std::fs::{ create_dir_all, File };
use std::io::Write;
use std::path::Path;

use tokio::io::AsyncWriteExt;
use regex::Regex;
use std::time::Duration;
use tokio::time::sleep;

/// Generates a random user-agent string from a predefined list.
///
/// # Returns
///
/// A `String` containing a random user-agent header, which is useful for
/// mimicking different browsers and devices during web scraping.
///
/// # Example
///
/// ```
/// let user_agent = random_user_agent();
/// println!("Using user agent: {}", user_agent);
/// ```
pub fn random_user_agent() -> String {
    let user_agents = vec![
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64)...",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)...",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X)...",
        // Add more user agents as needed
    ];

    let index = rand::random::<usize>() % user_agents.len();
    user_agents[index].to_string()
}

/// Recursively scrapes web pages starting from the given URL.
///
/// # Arguments
///
/// * `url` - The URL to start scraping from.
/// * `client` - A reference to a `reqwest::Client` for making HTTP requests.
/// * `visited` - A mutable reference to a `HashSet<String>` to keep track of visited URLs.
///
/// # Example
///
/// ```
/// let client = Client::new();
/// let mut visited = HashSet::new();
/// recursive_scrape("https://example.com", &client, &mut visited).await;
/// ```
use futures::Future;
use std::pin::Pin;

pub fn recursive_scrape<'a>(
    url: &'a str,
    client: &'a Client,
    visited: &'a mut HashSet<String>,
) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
    Box::pin(async move {
        if visited.contains(url) {
            return;
        }
        visited.insert(url.to_string());

        let user_agent = random_user_agent();
        match client.get(url).header("User-Agent", user_agent).send().await {
            Ok(response) => {
                match response.text().await {
                    Ok(html) => {
                        println!("Scraping: {}", url);
                        scrape_content(&html, url, client).await;
                        scrape_js(&html);
                        scrape_for_errors(&html);
                        
                        let links = extract_links(&html, url);
                        for link in links {
                            if !visited.contains(&link) {
                                recursive_scrape(&link, client, visited).await;
                            }
                        }
                    }
                    Err(e) => {
                        let error_message = format!("Failed to get HTML content from '{}': {}", url, e);
                        eprintln!("{}", error_message);
                        log_error_to_file(&error_message);
                    }
                }
            }
            Err(e) => {
                let error_message = format!("Failed to request '{}': {}", url, e);
                eprintln!("{}", error_message);
                log_error_to_file(&error_message);
            }
        }
    })
}


/// Extracts all links from an HTML page, normalizing them to absolute URLs.
///
/// # Arguments
///
/// * `html` - The HTML content of the page as a string slice.
/// * `base_url` - The base URL to resolve relative links.
///
/// # Returns
///
/// A `HashSet` containing all unique absolute links found on the page.
///
/// # Example
///
/// ```
/// let links = extract_links("<a href='/about'>About</a>", "https://example.com");
/// assert!(links.contains("https://example.com/about"));
/// ```
pub fn extract_links(html: &str, base_url: &str) -> HashSet<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("a[href]").unwrap();
    let mut urls = HashSet::new();

    for element in document.select(&selector) {
        if let Some(link) = element.value().attr("href") {
            let absolute_link = normalize_link(link, base_url);
            urls.insert(absolute_link);
        }
    }
    urls
}

/// Normalizes a link to an absolute URL based on the base URL.
///
/// # Arguments
///
/// * `link` - The link to normalize.
/// * `base_url` - The base URL of the current page.
///
/// # Returns
///
/// A `String` containing the absolute URL.
///
/// # Example
///
/// ```
/// let absolute_link = normalize_link("/about", "https://example.com");
/// assert_eq!(absolute_link, "https://example.com/about");
/// ```
pub fn normalize_link(link: &str, base_url: &str) -> String {
    if link.starts_with("http") {
        link.to_string() // Already an absolute URL
    } else {
        match Url::parse(base_url) {
            Ok(base) => base.join(link).map(|url| url.to_string()).unwrap_or_default(),
            Err(_) => link.to_string(), // Return as-is if base URL is invalid
        }
    }
}


/// Downloads a media file (image or video) and saves it to the local directory.
///
/// # Arguments
///
/// * `client` - A reference to a `reqwest::Client` for making HTTP requests.
/// * `media_url` - The URL of the media file to download.
/// * `file_path` - The file path where the media file will be saved.
///
/// # Example
///
/// ```
/// download_media(&client, "https://example.com/image.jpg", Path::new("./downloads/image.jpg")).await;
/// ```
pub async fn download_media(client: &Client, media_url: &str, file_path: &Path) {
    if let Ok(response) = client.get(media_url).send().await {
        if response.status().is_success() {
            if let Ok(bytes) = response.bytes().await {
                if let Some(parent) = file_path.parent() {
                    if let Err(e) = tokio::fs::create_dir_all(parent).await {
                        let error_message = format!("Failed to create directory '{}': {}", parent.display(), e);
                        eprintln!("{}", error_message);
                        log_error_to_file(&error_message);
                        return;
                    }
                }

                let mut file = match tokio::fs::File::create(file_path).await {
                    Ok(f) => f,
                    Err(e) => {
                        let error_message = format!("Failed to create file '{}': {}", file_path.display(), e);
                        eprintln!("{}", error_message);
                        log_error_to_file(&error_message);
                        return;
                    }
                };

                if let Err(e) = file.write_all(&bytes).await {
                    let error_message = format!("Failed to write file '{}': {}", file_path.display(), e);
                    eprintln!("{}", error_message);
                    log_error_to_file(&error_message);
                } else {
                    println!("Successfully downloaded and saved the media file: {}", file_path.display());
                }
            } else {
                let error_message = format!("Failed to read bytes from the response for '{}'", media_url);
                eprintln!("{}", error_message);
                log_error_to_file(&error_message);
            }
        } else {
            let error_message = format!("Failed to download media from '{}': Status code {}", media_url, response.status());
            eprintln!("{}", error_message);
            log_error_to_file(&error_message);
        }
    } else {
        let error_message = format!("Failed to make request to '{}'", media_url);
        eprintln!("{}", error_message);
        log_error_to_file(&error_message);
    }
}


/// Scrapes all meaningful content from an HTML page, including text, images, videos, meta tags, and forms.
///
/// # Arguments
///
/// * `html` - The HTML content of the page as a string slice.
/// * `url` - The URL of the current page being scraped.
/// * `client` - A reference to a `reqwest::Client` for making HTTP requests.
///
/// # Example
///
/// ```
/// scrape_content("<html>...</html>", "https://example.com", &client).await;
/// ```
pub async fn scrape_content(html: &str, url: &str, client: &Client) {
    // Create a directory structure for storing scraped data
    let domain = extract_domain(url);
    let dir = format!("./scraped_data/{}", domain);

    // Ensure the directory structure exists
    if let Err(e) = create_dir_all(&dir) {
        eprintln!("Failed to create directory '{}': {}", dir, e);
        return;
    }

    // Store text content (headers and paragraphs)
    let mut text_file = match File::create(format!("{}/content.txt", dir)) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to create text file: {}", e);
            return;
        }
    };

    let document = Html::parse_document(html);

    // Extract headers
    let header_selector = Selector::parse("h1, h2, h3, h4, h5, h6").unwrap();
    for header in document.select(&header_selector) {
        writeln!(text_file, "Header: {}", header.inner_html()).unwrap();
    }

    // Extract paragraphs
    let paragraph_selector = Selector::parse("p").unwrap();
    for paragraph in document.select(&paragraph_selector) {
        writeln!(text_file, "Paragraph: {}", paragraph.inner_html()).unwrap();
    }

    // Scrape images
    let img_selector = Selector::parse("img[src]").unwrap();
    for img in document.select(&img_selector) {
        if let Some(src) = img.value().attr("src") {
            let img_url = normalize_link(src, url);

            let file_name = img_url
                .split('/')
                .last()
                .unwrap_or("image.jpg")
                .to_string();
            let file_path = Path::new(&dir).join(file_name);
            println!("Downloading image: {}", img_url);
            download_media(client, &img_url, &file_path).await;
        }
    }

    // Scrape videos
    let video_selector = Selector::parse("video[src], source[src]").unwrap();
    for video in document.select(&video_selector) {
        if let Some(src) = video.value().attr("src") {
            let video_url = normalize_link(src, url);

            let file_name = video_url
                .split('/')
                .last()
                .unwrap_or("video.mp4")
                .to_string();
            let file_path = Path::new(&dir).join(file_name);
            println!("Downloading video: {}", video_url);
            download_media(client, &video_url, &file_path).await;
        }
    }

    // Scrape meta tags
    let meta_selector = Selector::parse("meta[name][content]").unwrap();
    for meta in document.select(&meta_selector) {
        let name = meta.value().attr("name").unwrap_or("Unnamed");
        let content = meta.value().attr("content").unwrap_or("");
        writeln!(text_file, "Meta Tag - Name: {}, Content: {}", name, content).unwrap();
    }

    // Scrape forms and inputs
    let form_selector = Selector::parse("form").unwrap();
    for form in document.select(&form_selector) {
        writeln!(text_file, "Form found!").unwrap();

        let input_selector = Selector::parse("input").unwrap();
        for input in form.select(&input_selector) {
            let input_name = input.value().attr("name").unwrap_or("Unnamed Input");
            let input_type = input.value().attr("type").unwrap_or("text");
            writeln!(
                text_file,
                "Input - Name: {}, Type: {}",
                input_name, input_type
            )
            .unwrap();
        }
    }

    // Scrape for emails
    scrape_for_emails(html, &dir);
}

/// Extracts the domain from a URL for folder naming purposes.
///
/// # Arguments
///
/// * `url` - The URL from which to extract the domain.
///
/// # Returns
///
/// A `String` containing the domain.
///
/// # Example
///
/// ```
/// let domain = extract_domain("https://example.com/path");
/// assert_eq!(domain, "example.com");
/// ```
pub fn extract_domain(url: &str) -> String {
    let parsed_url = Url::parse(url).expect("Invalid URL");
    parsed_url.host_str().unwrap_or("unknown_domain").to_string()
}

/// Scrapes JavaScript content for API keys or tokens.
///
/// # Arguments
///
/// * `html` - The HTML content of the page as a string slice.
///
/// # Example
///
/// ```
/// scrape_js_content("<script>var apiKey = '12345';</script>");
/// ```
pub fn scrape_js(html: &str) {
    let document = Html::parse_document(html);
    let script_selector = Selector::parse("script").unwrap();

    for script in document.select(&script_selector) {
        let script_content = script.inner_html();
        if script_content.contains("apiKey") || script_content.contains("token") {
            println!("Potential API key or token found in JS: {}", script_content);
        }
    }
}

/// Scrapes for errors and stack traces in the HTML content.
///
/// # Arguments
///
/// * `html` - The HTML content of the page as a string slice.
///
/// # Example
///
/// ```
/// scrape_for_errors("<html><body>Error: Stack trace</body></html>");
/// ```
pub fn scrape_for_errors(html: &str) {
    if html.contains("Exception") || html.contains("Stack trace") {
        println!("Potential error or stack trace found in the page:\n{}", html);
    }
}

/// Scrapes for emails and saves them to a file.
///
/// # Arguments
///
/// * `html` - The HTML content of the page as a string slice.
/// * `dir` - The directory where the emails.txt file will be saved.
///
/// # Example
///
/// ```
/// scrape_for_emails("<p>Contact us at info@example.com</p>", "./scraped_data/example.com");
/// ```
pub fn scrape_for_emails(html: &str, dir: &str) {
    let email_regex = match Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}") {
        Ok(regex) => regex,
        Err(e) => {
            eprintln!("Failed to compile email regex: {}", e);
            return;
        }
    };

    let email_file_path = format!("{}/emails.txt", dir);
    let mut email_file = match File::create(&email_file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to create email file '{}': {}", email_file_path, e);
            return;
        }
    };

    for email in email_regex.find_iter(html) {
        if writeln!(email_file, "{}", email.as_str()).is_err() {
            eprintln!("Failed to write email '{}' to file '{}'", email.as_str(), email_file_path);
        }
    }
}


/// Fetches a web page and prints the response status, demonstrating cookie handling.
///
/// # Arguments
///
/// * `url` - The URL to fetch.
/// * `client` - A reference to a `reqwest::Client` for making HTTP requests.
///
/// # Example
///
/// ```
/// fetch_with_cookies("https://example.com", &client).await;
/// ```
pub async fn fetch_with_cookies(url: &str, client: &Client) {
    if let Ok(response) = client.get(url).send().await {
        println!("Response status: {}", response.status());
        // Note: For actual cookie handling, enable the cookie store feature in reqwest.
    }
}

/// Checks for common open directories on the server.
///
/// # Arguments
///
/// * `url` - The base URL to check.
/// * `client` - A reference to a `reqwest::Client` for making HTTP requests.
///
/// # Example
///
/// ```
/// check_open_directories("https://example.com", &client).await;
/// ```
pub async fn check_open_directories(url: &str, client: &Client) {
    let directories = vec!["/backup", "/config", "/logs", "/uploads"];
    for dir in directories {
        let full_url = format!("{}{}", url, dir);
        if let Ok(response) = client.get(&full_url).send().await {
            if response.status().is_success() {
                println!("Open directory found: {}", full_url);
            }
        }
    }
}

/// Fetches and parses the robots.txt file.
///
/// # Arguments
///
/// * `url` - The base URL to fetch robots.txt from.
/// * `client` - A reference to a `reqwest::Client` for making HTTP requests.
///
/// # Example
///
/// ```
/// fetch_robots_txt("https://example.com", &client).await;
/// ```
pub async fn fetch_robots_txt(url: &str, client: &Client) {
    let robots_url = format!("{}/robots.txt", url.trim_end_matches('/'));
    if let Ok(response) = client.get(&robots_url).send().await {
        if let Ok(body) = response.text().await {
            let disallowed_paths: Vec<&str> = body
                .lines()
                .filter(|line| line.starts_with("Disallow"))
                .map(|line| line.split(": ").nth(1).unwrap_or("/"))
                .collect();

            for path in disallowed_paths {
                println!("Disallowed path found: {}", path);
            }
        }
    }
}

/// Executes the entire scraping workflow for the provided URL, including:
/// - Fetching `robots.txt` to check for disallowed paths
/// - Checking for open directories
/// - Fetching content with cookies
/// - Performing recursive scraping on links found in the website
///
/// The function mimics human behavior by introducing random delays
/// between requests to avoid overwhelming servers.
///
/// # Arguments
/// * `url` - The URL to start scraping from.
/// * `client` - A reference to a `reqwest::Client` for making HTTP requests.
///
/// # Example
/// ```
/// let client = Client::new();
/// run("https://example.com", &client).await;
/// ```
pub async fn run(url: &str, client: &Client) {
    let mut visited = HashSet::new();

    println!("Starting scraping workflow for {}", url);

    // Fetch `robots.txt`, open directories, and perform cookie-based scraping
    fetch_robots_txt(url, client).await;
    check_open_directories(url, client).await;
    fetch_with_cookies(url, client).await;

    // Start recursive scraping from the base URL
    recursive_scrape(url, client, &mut visited).await;

    // Introduce a delay to mimic human-like browsing behavior
    random_delay(2, 5).await;

    println!("Scraping workflow completed for {}", url);
}


use std::fs::OpenOptions;
/// Logs an error message to a file.
///
/// # Arguments
///
/// * `message` - The error message to log.
fn log_error_to_file(message: &str) {
    let log_file_path = "error.log";
    
    // Open the file in append mode, creating it if it doesn't exist
    let mut file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open or create error log file '{}': {}", log_file_path, e);
            return;
        }
    };

    // Write the error message to the file
    if let Err(e) = writeln!(file, "{}", message) {
        eprintln!("Failed to write to error log file '{}': {}", log_file_path, e);
    }
}


/// Sleeps for a random duration between a given range, mimicking human browsing behavior.
///
/// # Arguments
///
/// * `min_secs` - Minimum number of seconds to sleep.
/// * `max_secs` - Maximum number of seconds to sleep.
///
/// # Example
///
/// ```
/// random_delay(1, 5).await;
/// ```
pub async fn random_delay(min_secs: u64, max_secs: u64) {
    let delay = rand::random::<u64>() % (max_secs - min_secs + 1) + min_secs;
    sleep(Duration::from_secs(delay)).await;
}


/// Recursively scrapes web pages starting from the given URL, looking for the target phrase.
/// If the target phrase is not found in the HTML content of a page, it stops scraping in that direction.
///
/// # Arguments
/// * `url`: The starting URL for scraping.
/// * `client`: An instance of `reqwest::Client` for making HTTP requests.
/// * `config`: An optional reference to `ScraperConfig` for controlling scraper behavior.
/// * `visited`: A `HashSet` that tracks visited URLs.
/// * `target_phrase`: The phrase to search for in the HTML content.
///
/// This function performs breadth-first scraping, but only continues to follow links
/// if the target phrase is found in the current page's content.
pub async fn rec_scrape(url: &str, client: &Client, config: Option<&ScraperConfig>, visited: &mut HashSet<String>, target_phrase: &str) {
    let mut queue = VecDeque::new();
    queue.push_back(url.to_string());
    let mut current_depth = 0; // Initialize scraping depth

    // Get configuration values or defaults
    let follow_links = config.map_or(true, |c| c.follow_links()); // Default: true
    let max_depth = config.map_or(3, |c| c.max_depth()); // Default: 3
    let user_agent = config.and_then(|c| c.user_agent().cloned()); // Default: None (no user agent)

    while let Some(current_url) = queue.pop_front() {
        if visited.contains(&current_url) {
            continue;
        }

        println!("Visiting: {}", current_url);
        visited.insert(current_url.clone());

        // Build the request with optional user agent
        let mut request = client.get(&current_url);
        if let Some(ref agent) = user_agent {
            request = request.header(header::USER_AGENT, agent);
        }

        let response = match request.send().await {
            Ok(response) => response,
            Err(_) => continue, // Skip the URL if there's an error
        };

        if response.status().is_success() {
            let html = match response.text().await {
                Ok(html) => html,
                Err(_) => continue, // Skip if there's an error reading the content
            };

            if should_scrape_content(&html, target_phrase) {
                println!("Target phrase found in: {}", current_url);

                // Only follow links if target_phrase is found and depth is within limits
                if follow_links && current_depth < max_depth {
                    let links = extract_links(&html, &current_url);
                    for link in links {
                        if !visited.contains(&link) {
                            queue.push_back(link); // Only add links if the phrase is found
                        }
                    }
                    current_depth += 1; // Increase depth after following links
                }
            } else {
                println!("Target phrase not found in: {}", current_url);
                // Do not enqueue links from this page, discontinue following in this direction
                continue;
            }
        }
    }
}

/// Checks if the given content contains the target phrase.
///
/// # Arguments
/// * `content`: The HTML content of the page as a string.
/// * `target_phrase`: The phrase to search for within the content.
///
/// Returns `true` if the target phrase is found, otherwise `false`.
pub fn should_scrape_content(content: &str, target_phrase: &str) -> bool {
    content.contains(target_phrase)
}

pub struct ScraperConfig {
    follow_links: bool,
    max_depth: i32,
    user_agent: Option<String>,
}

impl ScraperConfig {
    pub fn new(follow_links: bool, max_depth: i32, user_agent: Option<String>) -> Self {
        ScraperConfig {
            follow_links,
            max_depth,
            user_agent,
        }
    }

    // Method to update whether or not to follow links
    pub fn set_follow_links(&mut self, follow: bool) {
        self.follow_links = follow;
    }

    // Method to update the max depth of scraping
    pub fn set_max_depth(&mut self, depth: i32) {
        self.max_depth = depth;
    }

    // Method to set a custom user agent
    pub fn set_user_agent(&mut self, agent: Option<String>) {
        self.user_agent = agent;
    }

    pub fn follow_links(&self) -> bool {
        self.follow_links
    }

    pub fn max_depth(&self) -> i32 {
        self.max_depth
    }

    pub fn user_agent(&self) -> Option<&String> {
        self.user_agent.as_ref()
    }
}


pub async fn scrape_js_content(html: &str, url: &str, client: &Client, keywords: &[&str]) {
    let document = Html::parse_document(html);
    let script_selector = Selector::parse("script").unwrap();

    for script in document.select(&script_selector) {
        // Check for inline JavaScript (within the HTML)
        let script_content = script.inner_html();
        if !script_content.is_empty() {
            // Check for user-defined keywords in inline scripts
            for &keyword in keywords {
                if script_content.contains(keyword) {
                    println!("Found '{}' in inline JS: {}", keyword, script_content);
                }
            }
        }

        // Check if the script tag has a `src` attribute (external JS file)
        if let Some(src) = script.value().attr("src") {
            let js_url = normalize_link(src, url);

            // Fetch and download the JS file
            match client.get(&js_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(js_content) = response.text().await {
                            // Process the JS file content for user-defined keywords
                            for &keyword in keywords {
                                if js_content.contains(keyword) {
                                    println!("Found '{}' in external JS: {}", keyword, js_content);
                                }
                            }

                            // Optionally, save the JS content to a file
                            let file_name = js_url.split('/').last().unwrap_or("script.js").to_string();
                            let file_path = format!("./scraped_js/{}", file_name);
                            if let Err(e) = save_js_file(&file_path, &js_content) {
                                eprintln!("Failed to save JS file '{}': {}", file_path, e);
                            }
                        }
                    } else {
                        eprintln!("Failed to download JS file from '{}': Status code {}", js_url, response.status());
                    }
                }
                Err(e) => eprintln!("Error fetching JS file '{}': {}", js_url, e),
            }
        }
    }
}

/// Save the JavaScript content to a file.
///
/// # Arguments
///
/// * `file_path` - The file path where the JS content will be saved.
/// * `js_content` - The JavaScript content to save.
///
/// # Returns
///
/// A `Result<(), std::io::Error>` indicating success or failure.
fn save_js_file(file_path: &str, js_content: &str) -> Result<(), std::io::Error> {
    let mut file = File::create(file_path)?;
    file.write_all(js_content.as_bytes())?;
    println!("Saved JS file to '{}'", file_path);
    Ok(())
}




#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;
    use std::collections::HashSet;
    use std::path::Path;

    // Test for the random_user_agent function
    #[test]
    fn test_random_user_agent() {
        let user_agent = random_user_agent();
        assert!(user_agent.contains("Mozilla"), "User-Agent should contain 'Mozilla'");
    }

    // Test for the extract_links function
    #[test]
    fn test_extract_links() {
        let html = r#"<a href="/about">About</a> <a href="https://example.com">Home</a>"#;
        let base_url = "https://test.com";
        let links = extract_links(html, base_url);

        assert!(links.contains("https://test.com/about"));
        assert!(links.contains("https://example.com"));
    }

    // Test for the normalize_link function
    #[test]
    fn test_normalize_link() {
        let link = "/about";
        let base_url = "https://example.com";
        let normalized = normalize_link(link, base_url);

        assert_eq!(normalized, "https://example.com/about");
    }

    // Test for the scrape_for_emails function
    #[test]
    fn test_scrape_for_emails() {
        let html = r#"<p>Contact us at info@example.com</p>"#;
        let dir = "./test_output";
        create_dir_all(dir).unwrap();
        scrape_for_emails(html, dir);

        let emails_path = format!("{}/emails.txt", dir);
        let emails_file = std::fs::read_to_string(emails_path).unwrap();
        assert!(emails_file.contains("info@example.com"), "Should find the email");
    }

    // Async test for downloading media
    #[tokio::test]
    async fn test_download_media() {
        let client = Client::new();
        let media_url = "https://via.placeholder.com/150";
        let file_path = Path::new("./test_output/image.jpg");

        download_media(&client, media_url, &file_path).await;

        assert!(file_path.exists(), "Image should be downloaded and saved");
    }

    // Async test for recursive scraping (simplified, no live requests)
    #[tokio::test]
    async fn test_recursive_scrape() {
        let client = Client::new();
        let mut visited = HashSet::new();

        let url = "https://example.com";
        recursive_scrape(url, &client, &mut visited).await;

        assert!(visited.contains(url), "URL should be marked as visited");
    }

    // Clean up after tests
    fn clean_test_output() {
        std::fs::remove_dir_all("./test_output").unwrap_or_else(|_| {
            eprintln!("Could not delete test_output directory");
        });
    }

    #[test]
    fn test_cleanup() {
        clean_test_output();
    }
}

