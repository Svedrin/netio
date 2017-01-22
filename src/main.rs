#[deny(warnings)]

use std::io::prelude::*;
use std::io::Error;
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

extern crate rand;
use rand::Rng;

extern crate clap;
use clap::{Arg, App};


#[derive(Debug, Clone, Copy)]
enum State {
    Sender,
    Receiver
}

fn run_as_server(){
    let listener = TcpListener::bind("0.0.0.0:55455").unwrap();

    println!("TCP server listening.");

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection from {:?}.", stream.peer_addr().unwrap());
                match run_benchmark(stream, State::Receiver, State::Sender) {
                    Ok(_)    => println!("yay"),
                    Err(err) => println!("Erreur! {:?}", err)
                }
            }
            Err(e) => {
                println!("Got Errorness: {:?}", e);
            }
        }
    }
}

fn run_as_client(server_addr: String){
    let stream = TcpStream::connect((server_addr.as_str(), 55455)).unwrap();
    match run_benchmark(stream, State::Sender, State::Receiver) {
        Ok(_)    => println!("yay"),
        Err(err) => println!("Erreur! {:?}", err)
    }
}

fn run_benchmark(mut stream: TcpStream, phase1: State, phase2: State) -> Result<(), Error> {
    let pkt_sizes : [usize; 7] = [32, 64, 1024, 1492, 1500, 2048, 16384];
    let test_duration = Duration::new(5, 0);

    for cur_size in pkt_sizes.iter() {
        for cur_state in [phase1, phase2].iter() {
            let until = Instant::now() + test_duration;

            let mut transferred_data:usize = 0;
            let mut transferred_pkts:usize = 0;

            match cur_state {
                &State::Sender =>  {
                    let random_data = rand::thread_rng()
                        .gen_ascii_chars()
                        .take(*cur_size)
                        .collect::<String>();

                    println!("Sending data in packets of {:?} bytes.", cur_size);
                    while Instant::now() < until {
                        match stream.write(random_data.as_bytes()) {
                            Ok(res)  => transferred_data += res,
                            Err(err) => return Err(err)
                        }
                        match stream.flush() {
                            Ok(_)  => transferred_pkts += 1,
                            Err(err) => return Err(err)
                        }
                    }

                    println!("Transferred {:?} bytes in {:?} packets", transferred_data, transferred_pkts);
                },
                &State::Receiver => {
                    println!("Receiving data.");
                    let _ = stream.set_read_timeout(Some(Duration::new(1, 0)));
                    while Instant::now() < until {
                        match stream.read(&mut [0; 16384]) {
                            Ok(res)  => {
                                transferred_data += res;
                                transferred_pkts += 1;
                            },
                            Err(err) => return Err(err)
                        }
                    }

                    println!("Received {:?} bytes", transferred_data);
                }
            }
        }
    }
    return Ok( () )
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
