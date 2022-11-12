use std::net::TcpListener;
use clap::Parser;

mod args;

fn main() {
    let args = args::Args::parse();
    let address = format!("{}:{}", args.host, args.port);

    let listener = TcpListener::bind(&address).unwrap();
    println!("Serving on {}", address);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        println!("Connection established!");
    }
}
