use std::{env};
use std::env::Args;
use std::sync::Arc;
use futures::future::{join_all};
use reqwest::Client;
use tokio::task::JoinHandle;
use std::time::Instant;
use tokio::sync::Semaphore;

mod wsdc_tasks;
use wsdc_tasks::{Competitor, create_task, preflight_check};

#[derive(Debug)]
struct Config {
    profile: String,
    concurrent_tasks: usize,
}

impl Config {
    fn from_params(profile: String, concurrent_tasks: usize) -> Config {
        Config { profile, concurrent_tasks }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program_config = parse_args(env::args())?;

    match program_config.profile.as_str() {
        "full" => {
            let client = Client::new();
            let record_location = env::var("RECORD_DIRECTORY").unwrap();
            let wsdc_base = env::var("WSDC_URL").unwrap();
            let concurrent_tasks: usize = program_config.concurrent_tasks;

            let wsdc_autocomplete = format!("{}/autocomplete?q=*", wsdc_base);
            let competitors_list_start_time = Instant::now();
            let wsdc_competitors: Vec<Competitor> = preflight_check(wsdc_autocomplete, &client).await;
            let elapsed_time = Instant::now().duration_since(competitors_list_start_time);
            println!("Fetched full list with query \"*\" - Duration: {:.2}s", elapsed_time.as_secs_f64());

            println!("Starting download... (this may take a while) - errors will be printed as they occur");
            println!("Process is allowed to run {} concurrent tasks", concurrent_tasks);
            let tasks_start_time = Instant::now();

            let wsdc_find = format!("{}/find?q=", wsdc_base);

            let mut tasks: Vec<JoinHandle<Result<(), ()>>>= vec![];
            let mut total_dancers: u32 = 0;

            let semaphore = Arc::new(Semaphore::new(concurrent_tasks));
            for competitor in wsdc_competitors {
                if competitor.wscid.is_some() {
                    total_dancers += 1;
                    let number = competitor.wscid.unwrap();
                    let client = client.clone();
                    let wsdc_find = wsdc_find.clone();
                    let record_location = record_location.clone();
                    let permit = semaphore.clone().acquire_owned().await.unwrap();
                    let task = create_task(number, client, wsdc_find, record_location, permit);
                    tasks.push(task);
                }
            }
            println!("Found {} dancers", total_dancers);

            join_all(tasks).await;

            let elapsed_time = Instant::now().duration_since(tasks_start_time);
            println!("Task execution - Duration: {:.2}s", elapsed_time.as_secs_f64());
            println!("Done!");
            return Ok(())
        },
        _ => {
            println!("Usage: wsdc_db_sync <profile> <concurrent_tasks>");
            println!("Example: wsdc_db_sync full 30");
            return Ok(());
        }
    }
}

fn parse_args(mut args: Args) -> Result<Config, &'static str> {
    args.next(); // Skip the program name
    let profile: String = match args.next() {
        Some(arg) => arg,
        None => {
            println!("No profile specified - printing usage");
            "".to_string()
        },
    };

    let concurrent_tasks: usize = match args.next() {
        Some(arg) => arg.parse::<usize>().unwrap(),
        None => {
            println!("No concurrent tasks specified - using default of 30");
            30
        },
    };

    Ok(Config::from_params(profile, concurrent_tasks))
}