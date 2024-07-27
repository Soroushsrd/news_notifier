use scraper::{Html, Selector};
use notify_rust::Notification;
use reqwest;
use tokio;
use tokio::time::{sleep, Duration};
use std::collections::HashSet;
use std::io;
use url::Url;


#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let mut already_notified = HashSet::new();
    let mut urls_and_selectors = Vec::new();

    loop {
        let mut url = String::new();
        println!("Please enter the url you want to hear from.");
        println!("If you're finished, write: done");
        io::stdin()
            .read_line(&mut url)
            .expect("Unable to read the line");
        let url = url.trim();
        if url == "done" {
            break;
        }

        let mut selector = String::new();
        println!("Please enter the selector you want us to check: ");
        io::stdin()
            .read_line(&mut selector)
            .expect("Unable to read the line");
        let selector = selector.trim();

        urls_and_selectors.push((url.to_string(), selector.to_string()));
    }

    loop {
        for (url, selector) in &urls_and_selectors {
            match url_parser(url) {
                Ok(parsed_url) => {
                    let response = reqwest::get(url).await?;
                    let html_content = response.text().await?;
                    let document = Html::parse_document(&html_content);
                    let container_selector = Selector::parse(selector).unwrap();

                    let mut notifications = Vec::new();

                    for container in document.select(&container_selector) {
                        if let Some(link) = container.select(&Selector::parse("a").unwrap()).next() {
                            let href = link.value().attr("href").unwrap_or("No href attribute found");
                            if href.starts_with(&parsed_url) {
                                let text = link.text().collect::<Vec<_>>().join(" ").trim().to_string();
                                if text.len() > 8 {
                                    if !already_notified.contains(href) {
                                        notifications.push((text, href.to_string()));
                                        already_notified.insert(href.to_string());
                                    }
                                }
                            }
                        }
                    }

                    for (text, href) in notifications {
                        let notif = Notification::new()
                            .summary(&text)
                            .body(&href)
                            .icon("firefox")
                            .show();
                        match notif {
                            Ok(_) => println!("Notification with the title {} was sent. link:\n {}", &text, &href),
                            Err(e) => println!("Error was encountered {}", e),
                        }
                    }
                }
                Err(e) => println!("Error parsing URL: {}", e),
            }
        }
        sleep(Duration::from_secs(600)).await;
    }
}

fn url_parser(input_url: &str) -> Result<String, url::ParseError> {
    let url = Url::parse(input_url)?;
    let base_url = format!("{}://{}", url.scheme(), url.host_str().unwrap());
    Ok(base_url)
}