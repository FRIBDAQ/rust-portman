use portman::portpool::ports;
use portman::responder::responder;

fn main() {
    let mut pool = ports::PortPool::new(30000, 1000);

    // Allocation patterns:

    let p1 = pool
        .allocate("RingMaster", "fox")
        .expect("Allocation failed");
    let p2 = pool.allocate("Readout", "fox").expect("Allocation failed");

    print_usage(&mut pool);

    // Free:

    pool.free(p1.port()).unwrap();
    print_usage(&mut pool);

    pool.free(p2.port()).unwrap();
    print_usage(&mut pool);

    let result = responder::process_request("LIST", &mut pool);
    if !result.is_ok() {
        panic!("Should have been error.");
    }
}

fn print_usage(pool: &mut ports::PortPool) {
    println!(
        "{}", 
        responder::process_request("LIST", pool).unwrap()
    );
}
