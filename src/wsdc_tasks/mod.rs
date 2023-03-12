use serde::{Serialize, Deserialize};
use reqwest::Client;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::{OwnedSemaphorePermit};
use tokio::task::JoinHandle;

#[derive(Serialize, Deserialize, Debug)]
pub struct Competitor {
    name: String,
    pub wscid: Option<u32>,
}

pub async fn preflight_check(wsdc_autocomplete: String, client: &Client) -> Vec<Competitor> {
    let res = client.get(wsdc_autocomplete).send().await.unwrap().text().await.unwrap();
    let dancers: Vec<Competitor> = serde_json::from_str(res.as_str()).expect("Failed to parse JSON");
    dancers
}

pub fn create_task(number: u32, client: Client, wsdc_find: String, record_location: String, permit: OwnedSemaphorePermit) -> JoinHandle<Result<(), ()>> {
    tokio::spawn(async move {
        let wsdc_url = format!("{}{}", wsdc_find, number);
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
    })
}