use rouille::Request;
use rouille::Response;

fn main() {
    rouille::start_server("localhost:8080", move |request| Response::text("hello world"));
}
