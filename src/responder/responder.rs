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
                let _ = pool.free(p).is_ok(); // We can't really handle errors.
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
///
///    The return value is a Result<u16, String> decoded from the actual
/// raw server reply.
///
pub fn request_port(
    service_name: &str,
    user_name: &str,
    request: &mpsc::Sender<RequestMessage>,
) -> Result<u16, String> {
    let (reply_sender, reply_receiver) = mpsc::channel();

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
///
/// release_port
///     Release an allocated port.
///
/// - port is the port to release and
/// - request is the sender side of the channel on which we make requests
///    of the responder.
///
pub fn release_port(
    port: u16,
    request: &mpsc::Sender<RequestMessage>,
) -> Result<(), mpsc::SendError<RequestMessage>> {
    request.send(RequestMessage::FreePort(port))
}
/// get_allocations
///    Returns the vector of allocations (it's up to the caller to decide
/// how to format them).
///
/// ### Parameters:
///
/// -   request - channel along which the request will be done.
///
///  ### Returns:
///
///    Result<Vec<UsedPort>, String>
pub fn get_allocations(
    request: &mpsc::Sender<RequestMessage>,
) -> Result<Vec<ports::UsedPort>, String> {
    let (reply_sender, reply_receiver) = mpsc::channel();
    request
        .send(RequestMessage::ListAllocations(reply_sender))
        .unwrap();
    match reply_receiver.recv() {
        Ok(msg) => match msg {
            Ok(replyok) => {
                if let ReplyMessage::ListAllocations(allocs) = replyok {
                    Ok(allocs)
                } else {
                    Err(String::from("Invalid reply from port manager"))
                }
            }
            Err(msg) => Err(msg),
        },
        Err(msg) => Err(msg.to_string()),
    }
}
