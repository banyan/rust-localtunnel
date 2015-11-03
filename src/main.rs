extern crate hyper;

use hyper::Client;
use hyper::header::Connection;
use std::net::TcpListener;
use std::net::TcpStream;
use std::io::{Read, Write};

fn handle_client(mut conn_remote: TcpStream) -> () {
    let host = "localhost:3000";
    let mut conn_local = TcpStream::connect(host).unwrap();

    let mut buf = [0; 512];
    loop {
        let _ = match conn_remote.read(&mut buf) {
            Err(e) => panic!("Got an error: {}", e),
            Ok(m) => {
                if m == 0 {
                    break;
                }
                m
            },
        };

        match conn_local.write(&buf) {
            Err(_) => break,
            Ok(_) => continue,
        }
    }
}

fn main() {
    let client = Client::new();

    let mut res = client.get("http://localhost:1236/?new")
        .header(Connection::close())
        .send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    println!("Response: {}", body);

    let listener = TcpListener::bind("localhost:1236").unwrap();
    for stream in listener.incoming() {
        let s = stream.unwrap();
        handle_client(s);
    }
}
