use std::{env};
use std::fs::File;
use std::io::Write;
use std::ops::Add;
use std::string::ToString;
use reqwest::blocking::Client;
use serde::{Serialize, Deserialize};
use regex::Regex;
use rayon::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
struct Competitor {
    name: String,
    id: Option<u32>,
}

fn main() {
    let blocking_client = Client::new();
    let record_location = env::var("RECORD_DIRECTORY").unwrap();
    let wsdc_base = env::var("WSDC_URL").unwrap();

    let mut wsdc_autocomplete = String::from(wsdc_base.clone());
    wsdc_autocomplete = wsdc_autocomplete.add("/autocomplete?q=*");
    let wsdc_numbers = preflight_check(wsdc_autocomplete, &blocking_client);

    let mut wsdc_find = String::from(wsdc_base.clone());
    wsdc_find = wsdc_find.add("/find?q=");

    println!("Found {} dancers", wsdc_numbers.len());
    println!("Starting download... (this may take a while)");

    wsdc_numbers.par_iter()
        .enumerate()
        .for_each(|(_index,num)| {
            let mut wsdc_url = String::from(wsdc_find.clone());
            wsdc_url = wsdc_url.add(num.to_string().as_str());
            let response = blocking_client.post(wsdc_url).send().unwrap();
            let filename = format!("{}/{}.json", record_location, num);
            let mut file = File::create(filename).unwrap();
            file.write_all(response.text().unwrap().as_bytes()).unwrap();
        });

    println!("Done!");
}

fn preflight_check(wsdc_autocomplete: String, client: &Client) -> Vec<u32> {
    let res = client.get(wsdc_autocomplete).send().unwrap().text().unwrap();
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