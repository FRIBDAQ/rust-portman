use crate::portpool::ports;
use std::sync::mpsc;

/// ReplyMessage
///    Each RequestMessage has  corresponding reply message type
///    that's sent along the reply channel that's supplied  in
///    the message request (if provided).
///    The actual message sent to a channels is Result where Ok contains
///    a reply message and Err contains an error message string.
///    Note that FreePort requests don't need a reply.
///
pub enum ReplyMessage {
    AllocatePort(u16),
    ListAllocations(Vec<ports::UsedPort>),
}

type Reply = Result<ReplyMessage, String>;

/// RequestMessage
///    This enum defines the set of messages that can be sent
///  to us, the responder to perform operations.  There are three
///  operations currently provided:
///
///  *   AllocatePort - allocates a new port.
///  *   FreePort     - frees a port that's been allocated.
///  *   ListAllocations - Provides a list of all allocations:
///
pub enum RequestMessage {
    AllocatePort {
        service_name: String,
        user_name: String,
        reply_chan: mpsc::Sender<Reply>,
    },
    FreePort(u16),
    ListAllocations(mpsc::Sender<Reply>),
    Terminate,
}

///
/// responder
///    This handles the logic of getting a request, dispatching it
///    and sending the reply/result.
///    We are an infinite loop, intended to run in a thread:
///
///    *   base - port pool base port number.
///    *   num  - Number of ports to manage.
///    *   request_chan - channel over which the requests are received.
///
pub fn responder(base: u16, num: u16, request_chan: mpsc::Receiver<RequestMessage>) {
    let mut pool = ports::PortPool::new(base, num);
    loop {
        let request = request_chan.recv().unwrap();
        match request {
            RequestMessage::AllocatePort {
                service_name,
                user_name,
                reply_chan,
            } => {
                println!("Request service {} for user {}", service_name, user_name);
                match pool.allocate(&service_name, &user_name) {
                    Ok(alloc) => reply_chan
                        .send(Ok(ReplyMessage::AllocatePort(alloc.port())))
                        .unwrap(),
                    Err(msg) => reply_chan.send(Err(msg)).unwrap(),
                }
            }
            RequestMessage::FreePort(p) => {
                println!("Free Port {}", p);
            }
            RequestMessage::ListAllocations(reply_chan) => {
                println!("List allocations");
                reply_chan
                    .send(Ok(ReplyMessage::ListAllocations(vec![])))
                    .unwrap();
            }
            RequestMessage::Terminate => break,
        }
    }
}
///
/// request_port
///    Interacts with the service thread to obtain a new port.
/// This takes care of formatting and sending the request as well
/// as receiving the reply.
///
///   *  service_name   - Name of service to advertise.
///   *  user_name      - Name of user advertising service.
///   *  request        - Sender side of the request channel.
///   *  reply          -  pair containing sender/receiver ends of the reply.
///
///    The return value is a Result<u16, String> decoded from the actual
/// raw server reply.
///
pub fn request_port(
    service_name: &str,
    user_name: &str,
    request: &mpsc::Sender<RequestMessage>,
    reply: (mpsc::Sender<Reply>, mpsc::Receiver<Reply>),
) -> Result<u16, String> {
    let reply_receiver = reply.1;
    let reply_sender = reply.0;

    // Send the request:

    request
        .send(RequestMessage::AllocatePort {
            service_name: String::from(service_name),
            user_name: String::from(user_name),
            reply_chan: reply_sender,
        })
        .unwrap();

    // Get the reply:

    match reply_receiver.recv().unwrap() {
        Ok(msg) => match msg {
            ReplyMessage::AllocatePort(port) => Ok(port),
            _ => Err(String::from("Invalid reply message type")),
        },
        Err(msg) => Err(msg),
    }
}
