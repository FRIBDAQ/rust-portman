use crate::portpool::ports;

///
/// Contains code to respond to requests from the port manager.
/// does so by interacting with a port pool passed in as a
/// parameter.
/// Requests are a command line like entity that can be:
/// Each request generates a reply which is either:
///
/// FAIL textual reason for failure
///
/// or
///
/// OK  followed by request specific information.
///
/// Requests:
///
/// GIMME service user
///    Requests  a port to be used to provide/advertise the
///    service 'service' for the specified 'user'.
///    on success the reply is 'OK portnum' where portnum
///    is the port number allocated to that service.
///
/// LIST
///     Returns a string containing a list of port usages.
///     The result contains lines separated by \n.  The first line
///     is of the form 'OK n' where 'n' is the number of lines that
///     follow. Remaining lines are of the form:
///     number service user
///     where number is an allocated port number and service,
///     user are the service and user to which that port number
///     was allocated.
///
/// Note that the actual returns are a Result<String, String>
/// where OK resuts are an Ok result and FAIl results are an
/// Err result.
pub fn process_request(request: &str, pool: &mut ports::PortPool) -> Result<String, String> {
    let command_words = request.split_whitespace().collect::<Vec<&str>>();
    if command_words.len() < 1 {
        Err(String::from("FAIL No command in request"))
    } else {
        let command = command_words[0];
        match command {
            "LIST" => Ok(list(pool)),
            "GIMME" => allocate(&command_words, pool),
            _ => Err(String::from("FAIL Invalid request")),
        }
    }
}

//  list
//   Produce a list of the actual pool allocation.

fn list(pool: &ports::PortPool) -> String {
    let usage = pool.usage();
    let mut result = String::new();
    result += format!("OK {}\n", usage.len()).as_str();
    for p in &usage {
        result += format!("{}\n", p).as_str();
    }
    if usage.len() > 0 {
        result.pop(); // Get rid of extra trailing \n
    }
    result
}

// Actually do the allocation of a port.
fn allocate(command: &Vec<&str>, pool: &mut ports::PortPool) -> Result<String, String> {
    // there must be exactly three command words:

    if command.len() != 3 {
        Err(String::from(
            "FAIL GIMME request must have a service and user",
        ))
    } else {
        let allocation = pool.allocate(command[1], command[2]);
        match allocation {
            Ok(port_info) => Ok(report_allocation(port_info)),
            Err(reason) => Err(report_failure(&reason)),
        }
    }
}
// Report a port allocation:

fn report_allocation(allocation: ports::UsedPort) -> String {
    format!("OK {}", allocation.port())
}
// Report a port allocation failure - reall just need to prefix FAIL on the string:

fn report_failure(reason: &str) -> String {
    let mut result: String = String::from("FAIL ");
    result += reason;
    return result;
}
