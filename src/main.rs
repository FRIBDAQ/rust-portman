use clap::{App, Arg, SubCommand};
use portman::responder::responder;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::net;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::process;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

type RequestChannel = Arc<Mutex<mpsc::Sender<responder::RequestMessage>>>;
type Socket = Arc<Mutex<TcpStream>>;
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
//#[derive(Parser, Default, Debug)]
//#[clap(author="Author: Ron Fox", version="1.0", about="Purpose: NSCLDAQ Port manager", long_about = None)]

// We're going to use clap to fill in this struct.
// Note that in current versios of Clap, we can just annotate the struct
// and clap will do the rest.
// However at present, up to debian 11, we're restricted to 2.27.1 at highest
// and that's a tiny bit more cumbersome.
//
#[derive(Debug, Clone, Copy)]
struct Arguments {
    listen_port: u16,
    port_base: u16,
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

// Use clap to specify/process the command line arguments
// into an Arguments struct.
fn parse_arguments() -> Arguments {
    // set up the clap parser:

    let parser = App::new("portman")
        .version("1.0")
        .author("Ron Fox")
        .about("Rust replacement for NSCLDAQ port manager - does not need container")
        .arg(
            Arg::with_name("listen-port")
                .short("l")
                .long("listen-port")
                .value_name("PORTNUM")
                .help("Port number on which the port manager listens for connections")
                .takes_value(true)
                .default_value("30000"),
        )
        .arg(
            Arg::with_name("port-base")
                .short("p")
                .long("port-base")
                .value_name("BASE")
                .help("Base of the port pool portman pmanagers")
                .takes_value(true)
                .default_value("31000"),
        )
        .arg(
            Arg::with_name("num-ports")
                .short("n")
                .long("num-ports")
                .value_name("NUM")
                .help("Number of ports portman manages")
                .takes_value(true)
                .default_value("1000"),
        )
        .get_matches();

    // Default parameter values:

    let mut result = Arguments {
        listen_port: 30000,
        port_base: 31000,
        num_ports: 1000,
    };

    // Use clap's parser override the default values.

    if let Some(listen) = parser.value_of("listen-port") {
        if let Ok(listen_value) = listen.parse::<u16>() {
            result.listen_port = listen_value;
        } else {
            eprintln!("The listen port value must be a 16 bit unsigned integer");
            process::exit(-1);
        }
    };

    if let Some(base) = parser.value_of("port-base") {
        if let Ok(base_value) = base.parse::<u16>() {
            result.port_base = base_value;
        } else {
            eprintln!("The port-base value must be a 16 bit unsigned integer");
            process::exit(-1);
        }
    }

    if let Some(num) = parser.value_of("num-ports") {
        if let Ok(num_value) = num.parse::<u16>() {
            result.num_ports = num_value;
        } else {
            eprintln!("The num-ports value must be a 16 bit unsigned integer");
            process::exit(-1);
        }
    }

    // return the parsed parameters.
    result
}

fn main() {
    let args = parse_arguments();
    println!("{:#?}", args);

    // Create the request channel and start the resopnder.

    let (request_send, request_receive) = mpsc::channel();
    let safe_req = Arc::new(Mutex::new(request_send));
    let service_handle = thread::spawn(move || {
        responder::responder(args.port_base, args.num_ports, request_receive)
    });

    // Now turn ourselves into a TCP/IP server that's
    // processing client requests.

    let server = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], args.listen_port))).unwrap();

    for request in server.incoming() {
        if let Ok(socket) = request {
            let safe_socket = Arc::new(Mutex::new(socket));
            process_request(&safe_req, &safe_socket);
        } else {
            // Fill in failure code here when we can figure out
            // what it should look like.
        }
    }
}

//  Given a connected socket, returns the line of text
//  received from it.  WE don't really havfe
fn read_request_line(socket: &Socket) -> String {
    let mut line: Vec<u8> = vec![];
    let so = socket.lock().unwrap();
    let mut reader = BufReader::new(so.try_clone().unwrap());
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

fn process_request(req_chan: &RequestChannel, so: &Socket) {
    so.lock()
        .unwrap()
        .set_read_timeout(Some(time::Duration::from_secs(10)))
        .unwrap(); // Limit time to request.
    println!("Connected from {:#?}", so.lock().unwrap().peer_addr());
    let request_line = read_request_line(so);
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
            create_allocation(
                Arc::clone(req_chan),
                Arc::clone(so),
                &service_name,
                &user_name,
            );
        }
        ClientRequest::List => {
            println!("Client requesting a list of port allocations");
            list_allocations(Arc::clone(req_chan), Arc::clone(so));
        }
        ClientRequest::Terminate => {
            println!("Client requesting shutdown");
            process::exit(0);
        }
        ClientRequest::Invalid => {
            println!("Client sent an invalid request");
            invalid_request(Arc::clone(so));
        }
    }
}

///
/// ## is_local
///
///   Determine if a socket is connected to a local peer.
///
fn is_local(so: &Socket) -> bool {
    let socket = so.lock().unwrap();
    if let Ok(peer) = socket.peer_addr() {
        if peer.is_ipv4() {
            peer.ip() == net::Ipv4Addr::new(127, 0, 0, 1)
        } else if peer.is_ipv6() {
            peer.ip() == net::Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)
        } else {
            false
        }
    } else {
        false
    }
}
//
//  Functions to process individual requests
//

///
/// ## invalid_request
///    Report that a request was invalid.
///
fn invalid_request(sock: Socket) {
    sock.lock()
        .unwrap()
        .write_all(String::from("FAIL - invalid request\n").as_bytes())
        .unwrap();
    sock.lock().unwrap().flush().unwrap();
}

///
/// ## list_allocations
///    Produce a list of allocations to the output socket.
///
fn list_allocations(req_chan: RequestChannel, so: Socket) {
    let allocations = responder::get_allocations(&req_chan.lock().unwrap()).unwrap();
    let mut sock = so.lock().unwrap();
    let result = sock.write_all(format!("OK {}\n", allocations.len()).as_bytes());
    if result.is_err() {
        return;
    }
    for aloc in allocations {
        let result = sock.write_all(format!("{}\n", aloc).as_bytes());
        if result.is_err() {
            return;
        }
    }
    let result = sock.flush();
    return;
}

///
/// ## create_allocation
///
///    Given an allocation request, allocates a port from the
///    service thread and spins off a thread to monitor the socket on which
///    the service was requested - when the socket becomes readable,
///    that thread drops the allocated port from the list of
///    allocated port.  This request is only allowed from local connections.
///
fn create_allocation(req_chan: RequestChannel, so: Socket, service: &str, user: &str) {
    if !is_local(&so) {
        let reply = String::from("FAIL can only allocate to local senders\n");
        so.lock().unwrap().write_all(reply.as_bytes()).unwrap();
        so.lock().unwrap().flush().unwrap();
    } else {
        let info = responder::request_port(service, user, &req_chan.lock().unwrap());
        match info {
            Ok(port) => {
                let reply = format!("OK {}\n", port);
                so.lock().unwrap().write_all(reply.as_bytes()).unwrap();
                so.lock().unwrap().flush().unwrap();
                thread::spawn(move || monitor_port(Arc::clone(&so), port, Arc::clone(&req_chan)));
            }
            Err(str) => {
                let reply = format!("FAIL {}\n", str);
                so.lock().unwrap().write_all(reply.as_bytes()).unwrap();
                so.lock().unwrap().flush().unwrap();
            }
        }
    }
}
//
// Monitor a socket so that its port can be released when
// the socket either closes or, alternatively,

fn monitor_port(socket: Socket, port: u16, req_chan: RequestChannel) {
    socket.lock().unwrap().set_read_timeout(None).unwrap(); // Turn off timeout
    let mut junk = String::new();
    if let Ok(n) = socket.lock().unwrap().read_to_string(&mut junk) {
    } else {
    }

    // Port is now closed so drop our side of the connection

    socket
        .lock()
        .unwrap()
        .shutdown(net::Shutdown::Both)
        .unwrap();

    responder::release_port(port, &req_chan.lock().unwrap()).unwrap();
}
