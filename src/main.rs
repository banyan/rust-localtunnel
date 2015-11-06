extern crate hyper;

use hyper::Client;
use hyper::header::Connection;
use std::net::TcpListener;
use std::net::TcpStream;
use std::net::Shutdown;
use std::io::{Read, Write};
use std::str;

fn handle_stream(mut conn_remote: TcpStream) -> () {
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

        let _ = match conn_local.write(&mut buf) {
            Err(e) => panic!("Got an error: {}", e),
            Ok(m) => {
                if m == 0 {
                    break;
                }
                m
            },
        };

        let mut buf : [u8; 512] = [0; 512];
        let _ = match conn_local.read(&mut buf) {
            Err(e) => panic!("Got an error: {}", e),
            Ok(m) => {
                if m == 0 {
                    break;
                }
                m
            },
        };

        // let foo = str::from_utf8(&buf).unwrap();
        // println!("Received {:?}", foo);

        let _ = match conn_remote.write(&buf) {
            Err(_) => break,
            Ok(m) => {
                if m == 0 {
                    break;
                }
                continue;
            },
        };
    }
    println!("a");
    conn_local.shutdown(Shutdown::Both);
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
        handle_stream(s);
    }
}
