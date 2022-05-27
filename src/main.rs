mod portpool;
use portpool::ports;
fn main() {
    
    let mut pool = ports::PortPool::new(30000, 1000);

    // Allocation patterns:

    let p1 = pool.allocate("RingMaster", "fox").expect("Allocation failed");
    let p2 = pool.allocate("Readout", "fox").expect("Allocation failed");

    print_usage(&pool);

    // Free:
    
    pool.free(p1.port()).unwrap();
    print_usage(&pool);

    pool.free(p2.port()).unwrap();
    print_usage(&pool);
    
}
fn print_usage(pool : &ports::PortPool) {
    println!("{}", get_usage(pool));
}

fn get_usage(pool : &ports::PortPool) -> String{
    let usage = pool.usage();
    let mut result =  String::new();
    result += format!("OK {}\n", usage.len()).as_str();
    for p  in &usage {
        result += format!("{}\n", p).as_str();
    }
    if usage.len() > 0 {
        result.pop();             // Get rid of extra trailing \n
    }
    result
}
