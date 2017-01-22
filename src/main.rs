#[deny(warnings)]

extern crate clap;
use clap::{Arg, App};

use std::net::{TcpListener, TcpStream};


#[derive(Debug)]
enum State {
    Sender,
    Receiver
}

fn run_as_server(){
    let listener = TcpListener::bind("0.0.0.0:55455").unwrap();

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                run_benchmark(stream, State::Receiver, State::Sender);
            }
            Err(e) => {
                println!("Got Errorness: {:?}", e);
            }
        }
    }
}

fn run_as_client(server_addr: String){
    let stream = TcpStream::connect((server_addr.as_str(), 55455)).unwrap();
    run_benchmark(stream, State::Sender, State::Receiver);
}

fn run_benchmark(stream: TcpStream, phase1: State, phase2: State) {
    println!("{:?}", stream);
    println!("{:?}", phase1);
    println!("{:?}", phase2);

    let pkt_sizes = [256, 1024, 1492, 1500, 2048, 16384];

    for cur_size in pkt_sizes.iter() {
        println!("{}", cur_size);
    }
}



fn main() {
    let matches = App::new("netio")
        .version("0.1.0")
        .author("Michael Ziegler <diese-addy@funzt-halt.net>")
        .about("network throughput benchmark")
        .arg(Arg::with_name("server-mode")
            .short("s")
            .help("Run in server mode"))
        .arg(Arg::with_name("server-addr")
            .help("the server to connect to")
            .index(1))
        .get_matches();

    if matches.is_present("server-mode") {
        run_as_server();
    }
    else{
        run_as_client(String::from(matches.value_of("server-addr").unwrap()));
    }
}
