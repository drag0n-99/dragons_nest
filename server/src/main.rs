use warp::{Filter, Reply};

fn list_agents_handler() -> impl Reply {
    println!("IT WORKS");
    "Listing agents"
}
fn list_jobs_handler() -> impl Reply {
    println!("IT WORKS");
    "Listing jobs"
}
fn exec_cmd_handler() -> impl Reply {
    println!("IT WORKS");
    "Executing job"
}

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let list_agents_route = warp::path!("list_agents").map(|| list_agents_handler());

    let list_jobs_route = warp::path!("list_jobs").map(|| list_jobs_handler());

    let exec_cmd_route = warp::path!("exec_cmd").map(|| exec_cmd_handler());

    // // Match any request and return hello world!
    // let hello = warp::any().map(|| "Hello, World!");

    let get_routes = warp::get().and(list_agents_route.or(list_jobs_route));
    let post_routes = warp::post().and(exec_cmd_route);

    let routes = get_routes.or(post_routes);
    warp::serve(routes)
        .tls()
        // Specify the path to your certificate and key files.
        .cert_path("tls/cert.pem")
        .key_path("tls/key.rsa")
        .run(([127, 0, 0, 1], 3031))
        .await;

    // warp::serve(hello)
    // .run(([127, 0, 0, 1], 3030))
    // .await;
}
