use clap::Command;
use std::error::Error;

/* Create a string slice constant. The string slice will live in the read-only
section of memory */

// subcommand to list agents
const AGENTS: &str = "agents";


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Command::new(clap::crate_name!())
        .subcommand(Command::new(AGENTS))
        .about("List all agents")
        .get_matches();

    /* .is_some() checks for 'Some()' variant of Option<T> */
    if cli.subcommand_matches(AGENTS).is_some() {
        println!("Success!");
        let response = reqwest::get(":8000").await?;
        if response.status().is_success() {
            let body = response.text().await?;
            println!("Response Body: {}", body);
        } else {
            println!("Request failed with status: {}", response.status());
        }
    }

    Ok(())
}
