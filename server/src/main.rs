use serde::{Deserialize, Serialize};
//use serde_json::json;
use std::fs::{File, OpenOptions};
use std::io::{prelude::*, BufReader};
use warp::{Filter, Rejection, Reply};

#[derive(Deserialize)]
struct Agent {
    uuid: String,
}

#[derive(Deserialize)]
struct Job {
    agent_uuid: String,
    command: String,
}

#[derive(Serialize)]
struct Response {
    message: String,
}

// By specifying the return type as Result<impl Reply, Rejection>
// We are saying that the return type will be 'some' type that implements the
// Reply trait, this could be a String or warp::reply::json()
// The Err() variant will return a Rejection which is a type from warp that
// means a request failed
fn register_agent_handler(agent: Agent) -> impl Reply {
    let file_path = "agent_list";

    // Read the existing agent list from the file
    let file = match OpenOptions::new().read(true).open(file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file for reading: {}", e);
            let response = Response {
                message: "Error opening file for reading".to_string(),
            };
            return warp::reply::json(&response);
        }
    };
    let reader = BufReader::new(file);

    // Check if the agent ID already exists in the file
    let agent_exists = reader
        .lines()
        .any(|line| line.unwrap_or_default() == agent.uuid.to_string());

    if agent_exists {
        let response = Response {
            message: format!("Agent with ID {} already exists", agent.uuid),
        };
        return warp::reply::json(&response);
    }

    // Append the new agent to the file
    let mut file = match OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(file_path)
    {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file for appending: {}", e);
            let response = Response {
                message: "Error opening file for appending".to_string(),
            };
            return warp::reply::json(&response);
        }
    };

    let content = format!("{}\n", agent.uuid);
    match file.write_all(content.as_bytes()) {
        Ok(_) => {
            let response = Response {
                message: "Successfully registered agent".to_string(),
            };
            warp::reply::json(&response)
        }
        Err(e) => {
            eprintln!("Error writing to file: {}", e);
            let response = Response {
                message: "Error writing to file".to_string(),
            };
            warp::reply::json(&response)
        }
    }
}

/*
 * By saying that we are returning a impl Reply, for the Ok() variant of Result, we are
 * saying that we are returning any type that implements the 'Reply' trait
 *
 * We're sending back a json response using 'warp::reply::json'
 *
 * We're sending the 'Response' type we created that implements 'Serialize'
*/
fn exec_cmd_handler(job: Job) -> impl Reply {
    println!("IT WORKS");
    println!("{}", job.command);
    let response = Response {
        message: "Executing Job".to_string(),
    };
    warp::reply::json(&response)
}

#[tokio::main]
async fn main() {
    let list_agents_route = warp::path!("list_agents").and(warp::fs::file("agent_list"));
    let list_jobs_route = warp::path!("list_jobs").and(warp::fs::file("job_list"));
    let exec_cmd_route = warp::path!("exec_cmd")
        .and(warp::post())
        .and(warp::body::json())
        //.and_then(exec_cmd_handler);
        .map(|job: Job| exec_cmd_handler(job));
    let register_agent = warp::path!("register_agent")
        .and(warp::post())
        .and(warp::body::json())
        .map(|agent: Agent| register_agent_handler(agent));

    let get_routes = warp::get().and(list_agents_route.or(list_jobs_route));
    let post_routes = warp::post().and(exec_cmd_route.or(register_agent));
    let routes = get_routes.or(post_routes);

    warp::serve(routes)
        .tls()
        .cert_path("tls/cert.pem")
        .key_path("tls/key.rsa")
        .run(([127, 0, 0, 1], 3031))
        .await;
}
