extern crate hyper;

use std::io::Read;

use hyper::Client;
use hyper::header::Connection;

fn main() {
    let client = Client::new();

    let mut res = client.get("http://localhost:1235/?new")
        .header(Connection::close())
        .send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    println!("Response: {}", body);
}
