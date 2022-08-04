use portman::portpool::ports;
use portman::responder::responder;
use std::sync::mpsc;
use std::thread;

fn main() {
    let (request_send, request_recv) = mpsc::channel();

    let handle = thread::spawn(|| responder::responder(30000, 1000, request_recv));

    // now we make some requests and get some replies:

    let port = responder::request_port("my_service", "fox", &request_send);
    analyze_port(&port);

    let allocations = responder::get_allocations(&request_send);
    analyze_allocations(allocations);

    if let Ok(port_num) = port {
        responder::release_port(port_num, &request_send).unwrap();
    }

    let terminate = responder::RequestMessage::Terminate;
    request_send.send(terminate).unwrap();

    handle.join().unwrap();
}

fn analyze_port(p: &Result<u16, String>) {
    match p {
        Ok(p) => {
            println!("Port {} allocated", p);
        }
        Err(msg) => {
            println!("Port allocation error {}", msg);
        }
    }
}
fn analyze_allocations(allocs: Result<Vec<ports::UsedPort>, String>) {
    match allocs {
        Ok(info) => {
            println!("{} allocations:", info.len());
            for alloc in info {
                println!("{}", alloc);
            }
        }
        Err(msg) => println!("Failed to get ellocations: {}", msg),
    }
}
