use serde::{Deserialize, Serialize};
use std::collections::HashMap;
//use serde_json::json;
use std::fs::{File, OpenOptions};
use std::io::{prelude::*, BufReader};
use std::path::Path;
use warp::{Filter, Rejection, Reply};

#[derive(Serialize, Deserialize)]
struct Agent {
    uuid: String,
}

#[derive(Serialize, Deserialize)]
struct Job {
    agent_uuid: String,
    job_uuid: String,
    command: String,
    output: String,
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
fn register_agent(agent: Agent) -> impl Reply {
    let file_path = "agent_list";

    // check if the file exists ie is this the first agent registering
    let mut agents: HashMap<String, String> = if Path::new(file_path).exists() {
        // If the file does exist load the agents into the agents HashMap
        ///!unsafe unwrap
        let mut file = File::open(file_path).unwrap();
        let mut contents = String::new();
        ///!unsafe unwrap
        file.read_to_string(&mut contents).unwrap();

        ///!unsafe unwrap
        let agents: HashMap<String, String> = serde_json::from_str(&contents).unwrap();
        agents
    } else {
        let mut agents: HashMap<String, String> = HashMap::new();
        agents
    };

    // .insert() takes ownership of the value being pushed that's why the formatted reply comes before the push
    let reply = format!("Successfully registered agent {}", &agent.uuid);

    // Check if the agent is already registered, if not register them
    if !agents.contains_key(&agent.uuid) {
        agents.insert(agent.uuid, "".to_string());

        ///! unsafe unwrap
        let serialized_agents = serde_json::to_string(&agents).unwrap();
        ///! unsafe unwrap
        let mut file = File::create(file_path).unwrap();
        ///! unsafe unwrap
        file.write_all(serialized_agents.as_bytes()).unwrap();

        return warp::reply::json(&reply);
    }

    warp::reply::json(&"Agent already registered")
}

/*
 * By saying that we are returning a impl Reply, for the Ok() variant of Result, we are
 * saying that we are returning any type that implements the 'Reply' trait
 *
 * We're sending back a json response using 'warp::reply::json'
 *
 * We're sending the 'Response' type we created that implements 'Serialize'
*/
/*
The job store is a HashMap<String,HashMap<String,HashMap<String,String>>>
                         agent_uuid,     job_uuid,        job,   output
*/
fn store_job(job: Job) -> impl Reply {
    // First validate the job
    // Is the agent_uuid provided an agent we have access to
    let file_path = "agent_list";
    let agents: HashMap<String, String> = if Path::new(file_path).exists() {
        // If the file does exist load the agents into the agents HashMap
        ///!unsafe unwrap
        let mut file = File::open(file_path).unwrap();
        let mut contents = String::new();
        ///!unsafe unwrap
        file.read_to_string(&mut contents).unwrap();

        ///!unsafe unwrap
        let agents: HashMap<String, String> = serde_json::from_str(&contents).unwrap();
        agents
    } else {
        return warp::reply::json(&"No agents registered");
    };

    if !agents.contains_key(&job.agent_uuid) {
        return warp::reply::json(&"Invalid agent");
    }

    println!("Valid agent {}\n", job.agent_uuid);
    let file_path = "job_list";

    // Check if the file job_list exists (ie is this the first job)
    let mut jobs: HashMap<String, HashMap<String, HashMap<String, String>>> =
        if Path::new(file_path).exists() {
            // Load the job_list as the jobs HashMap
            ///!unsafe unwrap
            let mut file = File::open(file_path).unwrap();
            let mut contents = String::new();
            ///!unsafe unwrap
            file.read_to_string(&mut contents).unwrap();

            ///!unsafe unwrap
            let jobs: HashMap<String, HashMap<String, HashMap<String, String>>> =
                serde_json::from_str(&contents).unwrap();
            println!("Loaded jobs\n");
            jobs
        } else {
            let mut jobs: HashMap<String, HashMap<String, HashMap<String, String>>> =
                HashMap::new();
            println!("Created new jobs hashmap\n");
            jobs
        };

    println!("Creating entry in hashmap for job\n");
    // Create a new entry in the jobs HashMap
    // Enter the new job
    jobs.entry(job.agent_uuid)
        // If the second level HashMap doesn't exist, create it as the value for the key job.agent_uuid
        .or_insert_with(HashMap::new)
        .entry(job.job_uuid)
        // If the third level HashMap doesn't exist, create it as the value for the key job.job_uuid
        .or_insert_with(HashMap::new)
        .insert(job.command, job.output);

    println!("Writing jobs hashmap to file");
    // Write the jobs HashMap to the job_list file
    ///! unsafe unwrap
    let serialized_jobs = serde_json::to_string(&jobs).unwrap();
    ///! unsafe unwrap
    let mut file = File::create(file_path).unwrap();
    ///! unsafe unwrap
    file.write_all(serialized_jobs.as_bytes()).unwrap();

    warp::reply::json(&"Successfully stored job")
}

fn job_output(job: Job) -> impl Reply {
    let file_path = "job_list";

    println!("loading jobs for job_output\n");
    // Load job_list as the jobs HashMap
    let mut jobs: HashMap<String, HashMap<String, HashMap<String, String>>> =
        if Path::new(file_path).exists() {
            ///!unsafe unwrap
            let mut file = File::open(file_path).unwrap();
            let mut contents = String::new();
            ///!unsafe unwrap
            file.read_to_string(&mut contents).unwrap();

            ///!unsafe unwrap
            let jobs: HashMap<String, HashMap<String, HashMap<String, String>>> =
                serde_json::from_str(&contents).unwrap();
            jobs
        } else {
            return warp::reply::json(&"job_store file doesn't exist");
        };

    println!("Setting job output for job_list\n");
    // Set the output for the corresponding agent_uuid and job_uuid
    match jobs.get_mut(&job.agent_uuid) {
        Some(job_map) => match job_map.get_mut(&job.job_uuid) {
            Some(command_map) => {
                command_map.insert(job.command, job.output);
            }
            None => return warp::reply::json(&"invalid job_uuid"),
        },
        None => return warp::reply::json(&"invalid agent_uuid"),
    }

    // Write the jobs HashMap to the job_list file
    ///! unsafe unwrap
    let serialized_jobs = serde_json::to_string(&jobs).unwrap();
    ///! unsafe unwrap
    let mut file = File::create(file_path).unwrap();
    ///! unsafe unwrap
    file.write_all(serialized_jobs.as_bytes()).unwrap();

    warp::reply::json(&"Successfully stored output")
}

fn list_jobs(agent: Agent) -> impl Reply {
    let file_path = "job_list";

    println!("loading jobs for list_jobs()\n");
    // Load job_list as the jobs HashMap
    let mut jobs: HashMap<String, HashMap<String, HashMap<String, String>>> =
        if Path::new(file_path).exists() {
            ///!unsafe unwrap
            let mut file = File::open(file_path).unwrap();
            let mut contents = String::new();
            ///!unsafe unwrap
            file.read_to_string(&mut contents).unwrap();

            ///!unsafe unwrap
            let jobs: HashMap<String, HashMap<String, HashMap<String, String>>> =
                serde_json::from_str(&contents).unwrap();
            jobs
        } else {
            return warp::reply::json(&"job_store file doesn't exist");
        };

    println!("agent uuid: {}\n", agent.uuid);

    match jobs.get(&agent.uuid) {
        Some(agent_jobs) => {
            let mut commands: Vec<Job> = Vec::new();

            for (job_uuid, job_details) in agent_jobs {
                for (command, output) in job_details {
                    if output.is_empty() {
                        let job = Job {
                            agent_uuid: agent.uuid.clone(),
                            job_uuid: job_uuid.clone(),
                            command: command.clone(),
                            output: output.clone(),
                        };
                        commands.push(job);
                    }
                }
            }

            return warp::reply::json(&commands);
        }
        None => return warp::reply::json(&"invalid agent_uuid"),
    }
}

#[tokio::main]
async fn main() {
    let list_agents_route = warp::path!("list_agents").and(warp::fs::file("agent_list"));
    let list_jobs_route = warp::path!("list_jobs")
        .and(warp::get())
        .and(warp::body::json())
        .map(|agent: Agent| list_jobs(agent));
    let jobs_output_route = warp::path!("job_output")
        .and(warp::post())
        .and(warp::body::json())
        .map(|output: Job| job_output(output));
    let store_job_route = warp::path!("send_job")
        .and(warp::post())
        .and(warp::body::json())
        .map(|job: Job| store_job(job));
    let register_agent_route = warp::path!("register_agent")
        .and(warp::post())
        .and(warp::body::json())
        .map(|agent: Agent| register_agent(agent));

    let get_routes = warp::get().and(list_agents_route.or(list_jobs_route));
    let post_routes = warp::post().and(store_job_route.or(register_agent_route));
    let routes = get_routes.or(post_routes).or(jobs_output_route);

    warp::serve(routes)
        .tls()
        .cert_path("tls/cert.pem")
        .key_path("tls/key.rsa")
        .run(([127, 0, 0, 1], 3031))
        .await;
}
