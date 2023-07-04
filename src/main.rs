#![recursion_limit = "1024"]

extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate rand;

use std::io::prelude::*;
use std::io::ErrorKind;
use std::io::stdout;
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};
use clap::{Arg, App};
use rand::RngCore;

mod errors {
    error_chain! { }
}

use errors::*;

#[derive(Debug, Clone, Copy)]
enum State {
    Sender,
    Receiver
}

fn print_rate(bytes: u64, time: Duration, label: &str){
    let mut rate: f64 = (bytes as f64) / (time.as_secs() as f64) * 8.0;

    let suffixes = [ "", "k", "M", "G", "T", "P", "E", "Z", "Y" ];

    for suffix in &suffixes {
        if rate < 1000.0 {
            print!("{:>8.2} {}Bit/s {}", rate, suffix, label);
            return;
        }
        rate /= 1000.0;
    }

    println!("SUPERCALIFRAGILISTICEXPIALIDOCIOUS");
}

fn run_as_server(port: u16, once: bool) -> Result<()> {
    let listener = TcpListener::bind(format!(":::{}", port))
        .chain_err(|| "Could not start server")?;

    println!("TCP server listening on port {}.", port);

    // accept connections and process them
    for stream in listener.incoming() {
        let stream = stream.expect("Could not accept client connection");
        if let Ok(addr) = stream.peer_addr() {
            println!("New connection from {:?}.", addr);
        }
        println!();
        if let Err(err) = run_benchmark(stream, State::Receiver, State::Sender) {
            println!("\n");
            return Err(err).chain_err(|| "Benchmark run aborted");
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

fn run_as_client(server_addr: &str, port: u16) -> Result<()> {
    let stream = TcpStream::connect((server_addr, port))
        .chain_err(|| "Could not connect to server")?;

    if let Ok(addr) = stream.peer_addr() {
        println!("Connected to {:?}.", addr);
    }
    println!();
    if let Err(err) = run_benchmark(stream, State::Sender, State::Receiver) {
        println!("\n");
        return Err(err).chain_err(|| "Benchmark run aborted");
    } else {
        println!("\nTest finished.");
    }
    println!();
    Ok(())
}

fn run_benchmark(mut stream: TcpStream, phase1: State, phase2: State) -> Result<()> {
    let pkt_sizes : [usize; 7] = [32, 64, 1024, 1492, 1500, 2048, 16_384];
    let test_duration = Duration::new(5, 0);

    // Packet size  1k bytes:  2293.17 KByte/s Tx,  2354.97 KByte/s Rx.

    for cur_size in pkt_sizes {
        stream.set_nodelay(cur_size < 1000)
            .chain_err(|| "Could not set TCP NoDelay option")?;

        print!("Packet size {:>5} bytes:   ", cur_size);
        stdout().flush()
            .chain_err(|| "Could not flush")?;

        for cur_state in &[phase1, phase2] {
            let until = Instant::now() + test_duration;

            let mut transferred_data:u64 = 0;

            match *cur_state {
                State::Sender =>  {
                    stream.set_read_timeout(None)
                        .chain_err(|| "Could not disable read timeout")?;

                    let mut random_data = vec![0; 16_384];
                    rand::thread_rng().fill_bytes(&mut random_data);

                    while Instant::now() < until {
                        transferred_data += stream.write(&random_data[..cur_size])
                            .and_then(|res| Ok(res as u64))
                            .or_else(|err| {
                                // "Resource temporarily not available" can happen, ignore
                                if err.kind() == ErrorKind::WouldBlock {
                                    Ok(0)
                                } else {
                                    Err(err)
                                }
                            })
                            .chain_err(|| "Could not send data")?;
                        stdout().flush()
                            .chain_err(|| "Could not flush stdout")?;
                    }

                    print_rate(transferred_data, test_duration, "Tx    ");
                    stdout().flush()
                        .chain_err(|| "Could not flush")?;

                    // wait for the "done" response from peer
                    stream.read(&mut [0; 16_384])
                        .chain_err(|| "Could not read \"done\" response")?;
                },
                State::Receiver => {
                    stream.set_read_timeout(Some(Duration::new(1, 0)))
                        .chain_err(|| "Could not enable read timeout")?;

                    while Instant::now() < until {
                        transferred_data += stream.read(&mut [0; 16_384])
                            .and_then(|res| Ok(res as u64))
                            .or_else(|err| {
                                // "Resource temporarily not available" can happen, ignore
                                if err.kind() == ErrorKind::WouldBlock {
                                    Ok(0)
                                } else {
                                    Err(err)
                                }
                            })
                            .chain_err(|| "Could not receive data")?;
                    }

                    print_rate(transferred_data, test_duration, "Rx    ");
                    stdout().flush()
                        .chain_err(|| "Could not flush stdout")?;

                    // There may be some data still left in transit, so read() until there's
                    // nothing left and then tell the sender we're done

                    while let Ok(_) = stream.read(&mut [0; 16_384]) {}

                    stream.write(b"done")
                        .chain_err(|| "Could not send \"done\" response")?;
                }
            }
        }
        println!();
    }
    Ok(())
}

fn run(matches: clap::ArgMatches) -> Result<()> {
    let port = matches.value_of("port").unwrap_or("55455").parse::<u16>()
        .chain_err(|| "Port argument must be a number between 1 and 65535")?;

    if matches.is_present("server-mode") || matches.is_present("one-shot") {
        run_as_server(port, matches.is_present("one-shot"))
    }
    else {
        run_as_client(
            matches.value_of("server-addr")
                .ok_or("Need a server to connect to when running in client mode, see --help")?,
            port
        )
    }
}

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author("Michael Ziegler <diese-addy@funzt-halt.net>")
        .about("network throughput benchmark")
        .arg(Arg::with_name("server-mode")
            .short('s')
            .long("server")
            .help("Run in server mode"))
        .arg(Arg::with_name("one-shot")
            .short('1')
            .long("one-shot")
            .help("Run in server mode, only once"))
        .arg(Arg::with_name("port")
            .short('p')
            .long("port")
            .takes_value(true)
            .help("Port number to use [55455]"))
        .arg(Arg::with_name("server-addr")
            .help("the server to connect to (client mode only)")
            .index(1))
        .get_matches();

    if let Err(ref e) = run(matches) {
        eprintln!("error: {}", e);

        for e in e.iter().skip(1) {
            eprintln!("caused by: {}", e);
        }

        if let Some(backtrace) = e.backtrace() {
            eprintln!("backtrace: {:?}", backtrace);
        }

        ::std::process::exit(1);
    }
}
