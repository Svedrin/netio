#[deny(warnings)]

extern crate clap;
extern crate rand;

use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::io::stdout;
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};
use clap::{Arg, App};
use rand::Rng;

#[derive(Debug, Clone, Copy)]
enum State {
    Sender,
    Receiver
}

fn print_rate(bytes: u64, time: Duration, label: String){
    let mut rate: f64 = (bytes as f64) / (time.as_secs() as f64) * 8.0;

    let suffixes = [ "", "k", "M", "G", "T", "P", "E", "Z", "Y" ];

    for suffix in suffixes.iter() {
        if rate < 1000.0 {
            print!("{:>8.2} {}Bit/s {}", rate, suffix, label  );
            return;
        }
        rate /= 1000.0;
    }

    println!("SUPERCALIFRAGILISTICEXPIALIDOCIOUS");
}

fn run_as_server(port: u16, once: bool) -> Result<(), String> {
    let listener = TcpListener::bind(format!(":::{}", port))
        .or_else(|err| Err(format!("Could not start server: {}", err)))?;

    println!("TCP server listening on port {}.", port);

    // accept connections and process them
    for stream in listener.incoming() {
        let stream = stream.expect("Could not accept client connection");
        if let Ok(addr) = stream.peer_addr() {
            println!("New connection from {:?}.", addr);
        }
        println!();
        if let Err(err) = run_benchmark(stream, State::Receiver, State::Sender) {
            println!("\nBenchmark run aborted: {}", err);
        } else {
            println!("\nTest finished.");
        }
        println!();
        if once {
            break;
        }
    }
    Ok(())
}

fn run_as_client(server_addr: String, port: u16) -> Result<(), String> {
    let stream = TcpStream::connect((server_addr.as_str(), port))
        .or_else(|err| Err(format!("Could not connect to server: {}", err)))?;

    if let Ok(addr) = stream.peer_addr() {
        println!("Connected to {:?}.", addr);
    }
    println!();
    if let Err(err) = run_benchmark(stream, State::Sender, State::Receiver) {
        println!("\nBenchmark run aborted: {}", err);
    } else {
        println!("\nTest finished.");
    }
    println!();
    Ok(())
}

fn run_benchmark(mut stream: TcpStream, phase1: State, phase2: State) -> Result<(), Error> {
    let pkt_sizes : [usize; 7] = [32, 64, 1024, 1492, 1500, 2048, 16384];
    let test_duration = Duration::new(5, 0);

    // Packet size  1k bytes:  2293.17 KByte/s Tx,  2354.97 KByte/s Rx.

    for cur_size in pkt_sizes.iter() {
        stream.set_nodelay(*cur_size < 1000)?;

        print!("Packet size {:>5} bytes:   ", cur_size);
        stdout().flush()?;

        for cur_state in [phase1, phase2].iter() {
            let until = Instant::now() + test_duration;

            let mut transferred_data:u64 = 0;

            match cur_state {
                &State::Sender =>  {
                    stream.set_read_timeout(None)?;

                    let random_data = rand::thread_rng()
                        .gen_ascii_chars()
                        .take(*cur_size)
                        .collect::<String>();

                    while Instant::now() < until {
                        match stream.write(random_data.as_bytes()) {
                            Ok(res)  => transferred_data += res as u64,
                            Err(err) => {
                                // "Resource temporarily not available" can happen, ignore
                                if err.kind() != ErrorKind::WouldBlock {
                                    return Err(err)
                                }
                            }
                        }
                        stdout().flush()?;
                    }

                    print_rate(transferred_data, test_duration, String::from("Tx    "));
                    stdout().flush()?;

                    // wait for the "done" response from peer
                    stream.read(&mut [0; 16384])?;
                },
                &State::Receiver => {
                    stream.set_read_timeout(Some(Duration::new(1, 0)))?;

                    while Instant::now() < until {
                        match stream.read(&mut [0; 16384]) {
                            Ok(res)  => transferred_data += res as u64,
                            Err(err) => {
                                // "Resource temporarily not available" can happen, ignore
                                if err.kind() != ErrorKind::WouldBlock {
                                    return Err(err)
                                }
                            }
                        }
                    }

                    print_rate(transferred_data, test_duration, String::from("Rx    "));
                    stdout().flush()?;

                    // There may be some data still left in transit, so read() until there's nothing left
                    // and then tell the sender we're done

                    while let Ok(_) = stream.read(&mut [0; 16384]) {}

                    stream.write("done".as_bytes())?;
                }
            }
        }
        println!();
    }
    return Ok( () )
}

fn main() {
    let matches = App::new("netio")
        .version("0.3.2")
        .author("Michael Ziegler <diese-addy@funzt-halt.net>")
        .about("network throughput benchmark")
        .arg(Arg::with_name("server-mode")
            .short("s")
            .long("server")
            .help("Run in server mode"))
        .arg(Arg::with_name("one-shot")
            .short("1")
            .long("one-shot")
            .help("Run in server mode, only once"))
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .takes_value(true)
            .help("Port number to use [55455]"))
        .arg(Arg::with_name("server-addr")
            .help("the server to connect to (client mode only)")
            .index(1))
        .get_matches();

    let port = matches.value_of("port").unwrap_or("55455").parse::<u16>()
        .expect("Port argument must be a number between 1 and 65535");

    if matches.is_present("server-mode") || matches.is_present("one-shot") {
        if let Err(err) = run_as_server(port, matches.is_present("one-shot")) {
            println!("{}", err);
        };
    }
    else{
        if let Err(err) = matches.value_of("server-addr")
            .ok_or_else( || String::from("Need a server to connect to when running in client mode, see --help") )
            .and_then(|addr| run_as_client(String::from(addr), port))
        {
            println!("{}", err);
        }
    }
}
