use portman::responder::responder;
use std::sync::mpsc;
use std::thread;

fn main() {
    let (request_send, request_recv) = mpsc::channel();
    let reply = mpsc::channel();

    let handle = thread::spawn(|| responder::responder(30000, 1000, request_recv));

    // now we make some requests and get some replies:

    let port = responder::request_port("my_service", "fox", &request_send, reply);
    analyze_port(&port);

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
