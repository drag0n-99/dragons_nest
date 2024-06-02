use reqwest::{Client, Error};
//use serde::Serialize;
use serde_json::json;
use std::fs::File;
use std::io::{prelude::*, Read};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .use_rustls_tls()
        .build()?;

    let file_path = "agent_id";
    let mut uuid = String::new();

    match File::open(file_path) {
        Ok(mut file) => {
            println!("File already exists. Overwriting its contents.");
            println!("File exists. Reading its contents.");
            match file.read_to_string(&mut uuid) {
                Ok(_) => {
                    println!("Successfully read the file contents.");
                    println!("UUID string: {}", uuid);
                }
                Err(e) => {
                    println!("Failed to read the file: {}", e);
                }
            }
        }
        // Create the file and write the agent uuid to it
        Err(_) => {
            println!("File does not exist. Creating a new file.");
            uuid = Uuid::new_v4().to_string();
            println!("agent uuid: {}", uuid);
            match File::create(file_path) {
                Ok(mut file) => match file.write_all(uuid.as_bytes()) {
                    Ok(_) => println!("Successfully wrote to the file."),
                    Err(e) => println!("Failed to write to the file: {}", e),
                },
                Err(e) => {
                    println!("Failed to create file: {}", e);
                }
            }
        }
    }

    let body = json!({"uuid": uuid});
    let content = client
        .post("https://127.0.0.1:3031/register_agent")
        .json(&body)
        .send()
        .await?
        .text()
        .await?;

    println!("text: {content:?}");

    Ok(())
}
