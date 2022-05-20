pub mod  portpool;
fn main() {
    println!("Hello, world!");
    let mut pool = portpool::ports::PortPool::new(30000, 1000);

    let p = pool.allocate("RingMaster", "fox").expect("Allocation failed");
    println!("{}", p);
}
