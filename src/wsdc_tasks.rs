use serde::{Serialize, Deserialize};
use reqwest::Client;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
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

pub fn create_task(number: u32, client: Client, wsdc_find: String, record_location: String, permit: OwnedSemaphorePermit) -> JoinHandle<Result<(), u32>> {
    tokio::spawn(async move {
        let wsdc_url = format!("{}{}", wsdc_find, number);
        let response = client.post(wsdc_url).send().await.unwrap();
        if response.status().is_success() {
            let filename = format!("{}/{}.json", record_location, number);
            
            let exists = tokio::fs::metadata(filename.clone()).await.is_ok();
            let content = response.text().await.unwrap();
            if exists == true {
                let existing_content = read_string_from_file(filename.clone()).await;
                if existing_content != content {
                    overwrite_string_to_file(filename.clone(), content).await;
                } else {
                    append_string_to_file(filename.clone(), content).await;
                }
            } else {
                append_string_to_file(filename.clone(), content).await;
            }
            
            drop(permit);
            Ok(())
        } else {
            eprintln!("Failed to download {}", &number);

            drop(permit);
            Err(number)
        }
    })
}

async fn append_string_to_file(filename: String, content: String) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)
        .await
        .unwrap();

    file.write_all(content.as_bytes()).await.unwrap();
}

async fn overwrite_string_to_file(filename: String, content: String) {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(filename)
        .await
        .unwrap();

    file.write_all(content.as_bytes()).await.unwrap();
}

async fn read_string_from_file(filename: String) -> String {
    let mut file = OpenOptions::new()
        .read(true)
        .open(filename)
        .await
        .unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).await.unwrap();
    content
}