use tokio::io;
use tokio::net::TcpStream;
use tokio::prelude::*;

pub fn connect() {
    let addr = "127.0.0.1:6142".parse().unwrap();

    let client = TcpStream::connect(&addr)
        .and_then(|stream| {
            println!("Created stream...");

            io::write_all(stream, "hello world\n").then(|result| {
                println!("wrote to stream; success={:?}", result.is_ok());
                Ok(())
            })
        })
        .map_err(|err| {
            println!("connection error = {:?}", err);
        });

    tokio::run(client);
}