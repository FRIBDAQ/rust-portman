use clap::Parser;
use portman::responder::responder;
use std::io::BufRead;
use std::io::BufReader;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

//
// Clap is kind of nice... with a few directives and
// a struct it'll generate the code to do reasonable
// command line parsing.
// The annotations and structs below define  the options:
//
// - -l, --listen-port which has a value that defaults to 30000
//    and will set the value of listen_port in this struct when parsed.
// - -p --port-base which has a default value of 31000 sets the
//      value of the port_base member.
// - -n, --num-ports has the default value of 1000 and sets the
//       value of the num_ports member.
//
// An impl Arguments is also automatically generated that, when
// invoked will parse the command line and return an Arguments
// struct.
//
#[derive(Parser, Default, Debug)]
#[clap(author="Author: Ron Fox", version="1.0", about="Purpose: NSCLDAQ Port manager", long_about = None)]
struct Arguments {
    #[clap(short, long, default_value_t = 30000)]
    listen_port: u16,
    #[clap(short, long, default_value_t = 31000)]
    port_base: u16,
    #[clap(short, long, default_value_t = 1000)]
    num_ports: u16,
}
#[derive(Debug)]
enum ClientRequest {
    Gimme {
        service_name: String,
        user_name: String,
    },
    List,
    Terminate,
    Invalid,
}

fn main() {
    let args = Arguments::parse();
    println!("{:#?}", args);

    // Create the request channel and start the resopnder.

    let (request_send, request_receive) = mpsc::channel();
    let service_handle = thread::spawn(move || {
        responder::responder(args.port_base, args.num_ports, request_receive)
    });

    // Now turn ourselves into a TCP/IP server that's
    // processing client requests.

    let server = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], args.listen_port))).unwrap();

    for request in server.incoming() {
        if let Ok(socket) = request {
            process_request(&request_send, socket);
        } else {
            // Fill in failure code here when we can figure out
            // what it should look like.
        }
    }
}

//  Given a connected socket, returns the line of text
//  received from it.  WE don't really havfe
fn read_request_line(socket: &TcpStream) -> String {
    let mut line: Vec<u8> = vec![];
    let mut reader = BufReader::new(socket.try_clone().unwrap());
    if let Ok(count) = reader.read_until(b'\n', &mut line) {
        String::from_utf8_lossy(&line).trim_end().to_string()
    } else {
        String::from("") // Illegal request
    }
}

fn decode_request(request_line: &str) -> ClientRequest {
    let request_words: Vec<&str> = request_line.split_ascii_whitespace().collect::<Vec<&str>>();

    // Need a word:

    if request_words.len() >= 1 {
        match request_words[0] {
            "GIMME" => {
                if request_words.len() == 3 {
                    ClientRequest::Gimme {
                        service_name: request_words[1].to_string(),
                        user_name: request_words[2].to_string(),
                    }
                } else {
                    ClientRequest::Invalid
                }
            }
            "LIST" => ClientRequest::List,
            "TERMINATE" => ClientRequest::Terminate,
            _ => ClientRequest::Invalid,
        }
    } else {
        ClientRequest::Invalid
    }
}

fn process_request(req_chan: &mpsc::Sender<responder::RequestMessage>, socket: TcpStream) {
    /// socket.set_read_timeout();    // Limit time to request.
    println!("Connected from {:#?}", socket.peer_addr());
    let request_line = read_request_line(&socket);
    println!("Request: {}", request_line);
    let request = decode_request(&request_line);
    match request {
        ClientRequest::Gimme {
            service_name,
            user_name,
        } => {
            println!(
                "Client Requesting port for {} user {}",
                service_name, user_name
            );
        }
        ClientRequest::List => {
            println!("Client requesting a list of port allocations");
        }
        ClientRequest::Terminate => {
            println!("Client requesting shutdown");
        }
        ClientRequest::Invalid => {
            println!("Client sent an invalid request");
        }
    }
}

//
//  Functions to process individual requests
//


//
// Monitor a socket so that its port can be released when
// the socket either closes or, alternatively, 