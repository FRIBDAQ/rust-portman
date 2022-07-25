use portman::portpool::ports;
use portman::responder::responder;

fn main() {
    let mut pool = ports::PortPool::new(30000, 1000);

    // Allocation patterns:

    println!(
        "{}",
        responder::process_request("GIMME RingMaster fox", &mut pool).unwrap()
    );
    println!(
        "{}",
        responder::process_request("GIMME Readout fox", &mut pool).unwrap()
    );

    print_usage(&mut pool);

    let result = responder::process_request("LIST", &mut pool);
    if !result.is_ok() {
        panic!("Should have been error.");
    }
}

fn print_usage(pool: &mut ports::PortPool) {
    println!("{}", responder::process_request("LIST", pool).unwrap());
}
