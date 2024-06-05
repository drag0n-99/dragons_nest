use clap::{Arg, ArgAction::Set, Command};
use reqwest::{Client, Error};
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

#[derive(Serialize)]
struct Job {
    agent_uuid: String,
    job_uuid: String,
    command: String,
    output: String,
}

async fn list_agents(client: &Client) -> Result<(), Error> {
    let content = client
        .get("https://127.0.0.1:3031/list_agents")
        .send()
        .await?
        .text()
        .await?;

    ///!unsafe unwrap
    let agents: HashMap<String, String> = serde_json::from_str(&content).unwrap();

    println!("Keys in the agents HashMap:");
    for key in agents.keys() {
        println!("{}", key);
    }
    Ok(())
}

async fn list_jobs(client: &Client) -> Result<(), Error> {
    let content = client
        .get("https://127.0.0.1:3031/list_jobs")
        .send()
        .await?
        .text()
        .await?;

    println!("text: {content:?}");
    Ok(())
}

async fn request_job_output(client: &Client, job_request: &Job) -> Result<(), Error> {
    ///! Unsafe unwrap change
    loop {
        // Low perf you have to serialize on every loop
        let serialized_job = serde_json::to_string(&job_request).unwrap();
        let content = client
            .get("https://127.0.0.1:3031/get_job_output")
            .body(serialized_job)
            .send()
            .await?
            .text()
            .await?;
        let reply = format!("{}", content);
        println!("text: {reply}\n");
        if !content.to_string().contains("Output Pending") {
            break;
        }

        thread::sleep(Duration::from_secs(1));
    }
    Ok(())
}

async fn send_job(client: &Client, agent_uuid: &String, command: &String) -> Result<(Job), Error> {
    let job = Job {
        agent_uuid: agent_uuid.clone(),
        job_uuid: Uuid::new_v4().to_string(),
        command: command.clone(),
        output: "".to_string(),
    };

    ///! Unsafe unwrap change
    let serialized_job = serde_json::to_string(&job).unwrap();

    let content = client
        .post("https://127.0.0.1:3031/schedule_job")
        .body(serialized_job)
        .send()
        .await?
        .text()
        .await?;

    println!("text: {content:?}");
    println!("agent id: {}", agent_uuid);
    println!("cmd: {}", command);
    Ok((job))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .use_rustls_tls()
        .build()?;

    let matches = Command::new(clap::crate_name!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .subcommand(Command::new("list-agents").about("List all agents"))
        .subcommand(Command::new("list-jobs").about("List all jobs"))
        .subcommand(
            Command::new("exec")
                .about("Execute a command")
                .arg(
                    Arg::new("agent-uuid")
                        .short('a')
                        .long("agent-uuid")
                        .help("The agent id to execute the command on")
                        .action(Set)
                        .required(true),
                )
                .arg(
                    Arg::new("command")
                        .help("The command to execute, with its arguments.")
                        .required(true)
                        .action(Set)
                        .index(1),
                ),
        )
        .arg_required_else_help(true)
        .get_matches();

    // pub fn subcommand(&self) -> Option<(&str, &ArgMatches)>
    // Returns a tuple of two values the first is the Arg id, the second is a reference to the arguments and their values
    match matches.subcommand() {
        Some(("list-agents", _)) => list_agents(&client).await?,
        // Some(("list-jobs", _)) => println!("run jobs() function"),
        Some(("list-jobs", _)) => list_jobs(&client).await?,
        Some(("exec", sub_m)) => {
            // Parse the agent_uuid to be a u128
            let agent_uuid: &String = sub_m
                // The signature for .get_one is this pub fn get_one<T>(&self, id: &str) -> Option<&T>
                // This means that we have to specify the type that is to be returned
                // By specify String as the type we are saying that the return type is string
                // This makes sense because all user input from the console is a String
                .get_one::<String>("agent-uuid")
                .expect("Agent ID is required");

            let command: &String = sub_m
                .get_one::<String>("command")
                .expect("Command is required");

            let job = send_job(&client, agent_uuid, command).await?;
            println!(
                "run exec() function, agent_uuid is {} and command is {}",
                agent_uuid, command
            );
            request_job_output(&client, &job).await?;
        }
        _ => unreachable!("Error: No subcommand match"),
    }

    Ok(())
}
