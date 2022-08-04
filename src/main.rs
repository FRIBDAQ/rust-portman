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

enum ClientRequest {
    Gimme {
        service_name: String,
        user_name: String,
    },
    List,
    Terminate,
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
            process_request(socket);
        } else {
            // Fill in failure code here when we can figure out
            // what it should look like.
        }
    }
}

fn read_request_line(socket: TcpStream) -> String {
    let mut line: Vec<u8> = vec![];
    let mut reader = BufReader::new(socket.try_clone().unwrap());
    let count = reader.read_until(b'\n', &mut line).unwrap();

    String::from_utf8_lossy(&line).trim_end().to_string()
}

fn process_request(socket: TcpStream) {
    println!("Connected from {:#?}", socket.peer_addr());
    println!("Request: {}", read_request_line(socket));
}
