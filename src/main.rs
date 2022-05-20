mod portpool;
use portpool::ports;
fn main() {
    
    let mut pool = ports::PortPool::new(30000, 1000);

    let p = pool.allocate("RingMaster", "fox").expect("Allocation failed");
    println!("{}", p);

    let usage = pool.usage();
    println!("OK {}", usage.len());
    for p  in usage {
        println!("{}", p);
    }
}
