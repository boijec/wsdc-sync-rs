use std::{env};
use std::ops::Add;
use std::string::ToString;
use std::sync::Arc;
use futures::future::join_all;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use regex::Regex;
use tokio::task::JoinHandle;
use tokio::sync::Semaphore;
use std::time::Instant;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

#[derive(Serialize, Deserialize, Debug)]
struct Competitor {
    name: String,
    id: Option<u32>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let record_location = env::var("RECORD_DIRECTORY").unwrap();
    let wsdc_base = env::var("WSDC_URL").unwrap();

    let mut wsdc_autocomplete = String::from(&wsdc_base);
    wsdc_autocomplete = wsdc_autocomplete.add("/autocomplete?q=*");
    let wsdc_numbers = preflight_check(wsdc_autocomplete, &client).await;
    println!("Found {} dancers", wsdc_numbers.len());

    let mut wsdc_find = String::from(&wsdc_base);
    wsdc_find = wsdc_find.add("/find?q=");
    let semaphore = Arc::new(Semaphore::new(21));
    let mut tasks: Vec<JoinHandle<Result<(), ()>>>= vec![];
    let start_time = Instant::now();
    println!("Starting download... (this may take a while)");

    for number in wsdc_numbers {
        let client = client.clone();
        let wsdc_find = wsdc_find.clone();
        let record_location = record_location.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let task = tokio::spawn(async move {
            let mut wsdc_url = String::from(&wsdc_find);
            wsdc_url = wsdc_url.add(number.to_string().as_str());
            let response = client.post(wsdc_url).send().await.unwrap();
            if response.status().is_success() {
                let content = response.text().await.unwrap();
                let filename = format!("{}/{}.json", record_location, number);
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(filename)
                    .await
                    .unwrap();

                file.write_all(content.as_bytes()).await.unwrap();
            } else {
                eprintln!("Failed to download {}", number);
            }
            drop(permit);
            Ok(())
        });
        tasks.push(task);
    }

    join_all(tasks).await;

    let elapsed_time = Instant::now().duration_since(start_time);
    println!("Elapsed time: {:.2}s", elapsed_time.as_secs_f64());
    println!("Done!");
    Ok(())
}

async fn preflight_check(wsdc_autocomplete: String, client: &Client) -> Vec<u32> {
    let res = client.get(wsdc_autocomplete).send().await.unwrap().text().await.unwrap();
    let mut dancers: Vec<Competitor> = serde_json::from_str(res.as_str()).expect("Failed to parse JSON");
    dancers.remove(0);
    let mut wsdc_numbers: Vec<u32> = Vec::new();
    let re = Regex::new(r"\((\d+)\)").expect("Failed to compile regex");
    for dancer in dancers {
        if let Some(caps) = re.captures(dancer.name.as_str()) {
            let number_str = caps.get(1).unwrap().as_str();
            if let Ok(number) = number_str.parse::<u32>() {
                wsdc_numbers.push(number);
            }
        }
    }
    wsdc_numbers.sort();
    wsdc_numbers
}