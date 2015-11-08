extern crate hyper;

#[macro_use]
extern crate log;
extern crate env_logger;

use hyper::Client;
use hyper::header::Connection;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::time::Duration;
use std::io::BufWriter;
use std::thread;

fn handle_stream(mut remote_stream: TcpStream) -> () {
    let host = "localhost:3000";
    let mut local_stream = TcpStream::connect(host).unwrap();

    remote_stream.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
    local_stream.set_read_timeout(Some(Duration::from_millis(100))).unwrap();

    let mut buf = [0; 128];
    let mut written = 0;

    loop {
        let len = match remote_stream.read(&mut buf) {
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e)  => break,
            Ok(0)   => break,
            Ok(len) => len
        };
        debug!("Read {} bytes from remote stream", len);
        let _ = local_stream.write_all(&buf[..len]);
        written += len as u64;
        debug!("Wrote {} bytes to local stream", written);
    }

    let mut buf2 = [0; 128];
    written = 0;

    loop {
        let len = match local_stream.read(&mut buf2) {
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e)  => break,
            Ok(0)   => break,
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

fn main() {
    env_logger::init().unwrap();
    let client = Client::new();

    let mut res = client.get("http://localhost:1236/?new")
        .header(Connection::close())
        .send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    println!("Response: {}", body);

    let listener = TcpListener::bind("localhost:1236").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_stream(stream);
                });
            }
            Err(e) => panic!("Got an error: {}", e)
        }
    }
}
