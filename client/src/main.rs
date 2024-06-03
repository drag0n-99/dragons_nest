use clap::{Arg, ArgAction::Set, Command};
use reqwest::{Client, Error};
use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
struct Job {
    agent_uuid: String,
    command: String,
}

async fn list_agents_handler() -> Result<(), Error> {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .use_rustls_tls()
        .build()?;

    let content = client
        .get("https://127.0.0.1:3031/list_agents")
        .send()
        .await?
        .text()
        .await?;

    println!("text: {content:?}");
    Ok(())
}

async fn list_jobs_handler() -> Result<(), Error> {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .use_rustls_tls()
        .build()?;

    let content = client
        .get("https://127.0.0.1:3031/list_jobs")
        .send()
        .await?
        .text()
        .await?;

    println!("text: {content:?}");
    Ok(())
}

async fn exec_cmd_handler(agent_uuid: &String, command: &String) -> Result<(), Error> {
    let job = Job {
        agent_uuid: agent_uuid.clone(),
        command: command.clone(),
    };

    ///! Unsafe unwrap change
    let serialized_job = serde_json::to_string(&job).unwrap();

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .use_rustls_tls()
        .build()?;

    let content = client
        .post("https://127.0.0.1:3031/exec_cmd")
        .body(serialized_job)
        .send()
        .await?
        .text()
        .await?;

    println!("text: {content:?}");
    println!("agent id: {}", agent_uuid);
    println!("cmd: {}", command);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
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
        Some(("list-agents", _)) => list_agents_handler().await?,
        // Some(("list-jobs", _)) => println!("run jobs() function"),
        Some(("list-jobs", _)) => list_jobs_handler().await?,
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

            exec_cmd_handler(agent_uuid, command).await?;
            println!(
                "run exec() function, agent_uuid is {} and command is {}",
                agent_uuid, command
            );
        }
        _ => unreachable!("Exhaustive checking in subcommand match failed"),
    }

    Ok(())
}
