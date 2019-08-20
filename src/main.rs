use rouille::Request;
use rouille::Response;

use std::env;

fn main() {
    let port: String = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");
    
    rouille::start_server(format!("0.0.0.0:{}", port), move |request| Response::text("hello world"));
}
