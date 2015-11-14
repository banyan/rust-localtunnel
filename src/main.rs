extern crate hyper;

extern crate rustc_serialize;
use rustc_serialize::json;

#[macro_use]
extern crate log;
extern crate env_logger;

use hyper::Client;
use hyper::header::Connection;
use std::io::{self, Read, Write};
use std::net::{TcpStream, Shutdown};
use std::time::Duration;
use std::io::BufWriter;
use std::thread;
use std::fmt;

const READ_TIMEOUT_MILLIS: u64 = 100;

fn handle_stream(mut remote_stream: TcpStream, remote_host: String, x: i8) -> () {
    println!("{}", x);
    let local_host = "localhost:3000";
    let mut local_stream = TcpStream::connect(local_host).unwrap();

    debug!("establishing tunnel {} <> {}", remote_host, local_host);

    remote_stream.set_read_timeout(Some(Duration::from_millis(READ_TIMEOUT_MILLIS))).unwrap();
    local_stream.set_read_timeout(Some(Duration::from_millis(READ_TIMEOUT_MILLIS))).unwrap();

    let mut buf = [0; 512];
    let mut written = 0;

    loop {
        let len = match remote_stream.read(&mut buf) {
            Err(e) => {
                match e.kind() {
                    io::ErrorKind::WouldBlock => {
                        debug!("a: would have blocked: {:?}", e.kind());
                        if written != 0 {
                            // FIXME more better way to distinguish
                            break
                        } else {
                            continue
                        }
                    },
                    io::ErrorKind::Interrupted => {
                        debug!("a: interrupted: {:?}", e.kind());
                        continue
                    },
                    _ => panic!("a: Got an error: {}", e)
                }
            },
            Ok(0)   => break,
            Ok(len) => len
        };
        debug!("Read {} bytes from remote stream", len);
        let _ = local_stream.write_all(&buf[..len]);
        written += len as u64;
        debug!("Wrote {} bytes to local stream", written);
    }

    let mut buf2 = [0; 512];
    written = 0;

    loop {
        let len = match local_stream.read(&mut buf2) {
            Err(e) => {
                match e.kind() {
                    io::ErrorKind::WouldBlock => {
                        debug!("would have blocked");
                        break
                    },
                    io::ErrorKind::Interrupted => break,
                    _ => panic!("Got an error: {}", e)
                }
            },
            Ok(0)   => {
                debug!("hi2");
                break
            },
            Ok(len) => len
        };
        debug!("Read {} bytes from local stream", len);
        let _ = remote_stream.write_all(&buf2[..len]);
        written += len as u64;
        debug!("Wrote {} bytes to remote stream", written);
    }

    let _ = local_stream.shutdown(Shutdown::Both);
    let _ = remote_stream.shutdown(Shutdown::Both);
}

#[derive(Clone, RustcDecodable)]
pub struct AssignedUrl  {
    port: i32,
    max_conn_count: i8,
    id: String,
    url: String
}

impl fmt::Debug for AssignedUrl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "port: {:?}, url: {:?}, max_conn_count: {:?}, id: {:?}", self.port, self.url, self.max_conn_count, self.id)
    }
}

fn main() {
    env_logger::init().unwrap();
    let client = Client::new();

    let mut res = client.get("http://localtunnel.me/?new")
        .header(Connection::close())
        .send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    println!("Response: {}", body);

    let decoded: AssignedUrl = json::decode(&body).unwrap();

    debug!("Decoded: {:?}", decoded);
    println!("{}", decoded.url);

    let threads = (0..decoded.max_conn_count).map(|x| {
        let host = format!("{}:{}", "localtunnel.me", decoded.clone().port);
        thread::spawn(move || {
            let mut remote_stream = TcpStream::connect(&*host).unwrap();
            handle_stream(remote_stream, host, x);
        })
    }).collect::<Vec<_>>();

    for t in threads {
        let _ = t.join();
    }
}
