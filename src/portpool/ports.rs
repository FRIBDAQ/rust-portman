use std::collections::HashSet;  
use std::collections::HashMap;
use std::fmt;

// Contains definitions and implemntations for port pools.
// A port pool consists of a free set of ports and a used set
// of ports.  
// Used ports contain the port number, the port service
// name and the port username.
//

pub struct UsedPort {
    port_number : u16,
    port_service : String,
    port_user   :  String, 
}
impl UsedPort {
    pub fn new(n : u16, service : &str, user : &str) -> UsedPort {
        UsedPort {
            port_number : n,
            port_service : String::from(service),
            port_user    : String::from(user)
        }
    }
    pub fn port(&self) -> u16 {
        self.port_number
    }
    pub fn service(&self) -> String {
        String::from(self.port_service.as_str())
    }
    pub fn user(&self) -> String {
        String::from(self.port_user.as_str())
    }
}

// So we can produce a formatted UsedPort:
// Output format is the same as what is produced
// from the LIST requrest to the port manager:

impl fmt::Display for UsedPort {
    fn fmt(&self, f : &mut fmt::Formatter<'_> ) -> fmt::Result {
        write!(f, "{} {} {}", self.port(), self.service(), self.user())
    }
}

// Unused ports are just the port number:

type UnusedPort = u16;

// A port pool requires collections of both the available
// and used ports.

pub struct PortPool {
    used : HashMap<u16, UsedPort>,
    unused : HashSet<UnusedPort>
}

impl PortPool {
    ///
    /// Create a new port pool:
    ///  start is the starting port.  n is the number of ports in the pool.
    /// 
    pub fn new(start : u16, n : u16) -> PortPool {
        // Generate the unused port pool

        let mut unused = HashSet::new();
        for p in start..(start+n) {
            unused.insert(p);
        }
        PortPool {
            used : HashMap::new(),
            unused : unused
        }
    }
    // Mark 'port' as used.
    //
    fn mark_used(&mut self, port: u16) {
        self.unused.remove(&port);
    }
    // Return a port, any port that is not yet in use.
    //
    fn get_unused(&self) -> u16 {
        let pport = self.unused.iter().next().expect("Bug non-empty free port pool iterator failed");
        *pport
    }
    // Return true if there's an allocated port already with the service/user pair.
    //
    fn in_use(&self, service : &str, user : &str) -> bool {
        for (_, value) in self.used.iter() {
            if value.port_service == service && value.port_user == user {
                return true;
            }
        }
        return false;
    }
    ///
    /// Allocate a port from the pool.  The port will be advertised with the
    /// service name 'servie' qualified by the user 'user'.  The return value will be
    /// a UsedPort describing the allocated port on success or a failure reason string
    /// on failure.
    ///
    pub fn allocate(&mut self, service : &str, user: &str) -> Result<UsedPort, String> {
        if self.unused.len() == 0 {
            return Err(String::from("No free ports available"))
        } else {
            if self.in_use(service, user) {
                return Err(String::from ("Duplicate port allocation attempted"));
            }
            let  port = self.get_unused();
            
            self.mark_used(port);
            self.used.insert(port, UsedPort::new(port, service, user));
            Ok(UsedPort::new(port, service, user))
        }
    }
    ///
    /// return a vector of the used ports.
    /// 
    pub fn usage(&self) -> Vec<UsedPort> {
        let mut result : Vec<UsedPort> = Vec::new();
        for (_, value) in &self.used {
            let u = UsedPort::new(
                value.port(), value.service().as_str(), value.user().as_str()
            );
            result.push(u);
        }
        result.sort_by_key(|k| {k.port_number});
        result
    }
    ///
    ///  Given a used 'port' number return it to the unused port pool.
    ///  The result is eithert the original port number for Ok or an
    ///  a string describing the failure.
    /// 
    pub fn free(&mut self, port: u16) ->Result<u16, String>     {
        
        match self.used.remove(&port) {
            Some(_) => {
                self.unused.insert(port);
                Ok(port)
            }
            None => Err(String::from("Port is not allocated"))
        }
    }
}
//
// Unit tests:
//
#[cfg(test)]
mod tests {
    use super::*;


    // UsedPort type 

    #[test]
    fn uport_construct() {
        let u = UsedPort::new(100, "Mytest", "Fox");
        assert_eq!(u.port_number, 100);
        assert_eq!(u.port_service, String::from("Mytest"));
        assert_eq!(u.port_user, String::from("Fox"));
    }
    #[test]
    fn uport_port() {
        let u = UsedPort::new(100, "Mytest", "Fox");
        assert_eq!(100, u.port());
    }
    #[test]
    fn uport_service() {
        let u = UsedPort::new(100, "Mytest", "Fox");
        assert_eq!(String::from("Mytest"), u.service());
    }
    #[test]
    fn uport_user() {
        let u = UsedPort::new(100, "Mytest", "Fox");
        assert_eq!(String::from("Fox"), u.user());
    }

    // PortPool type:

    #[test]
    fn portpool_construct() {
        let pool = PortPool::new(1000, 10);
        assert_eq!(0, pool.used.len());
        assert_eq!(10, pool.unused.len());
    }
    #[test]
    fn portpool_allocate_1() {               // Success.
        let mut pool = PortPool::new(1000, 1);   // One port avail.
        let result = pool.allocate("Service", "fox");
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(1000, info.port_number);
        assert_eq!(String::from("Service"), info.port_service);
        assert_eq!(String::from("fox"), info.port_user);

    }
    #[test]
    fn portpool_allocate_2() {       // No free ports:
        let mut pool = PortPool::new(1000, 1);
        pool.allocate("Ok", "fox").unwrap();

        // This one should fail:

        let result = pool.allocate("Fails", "fox");
        assert!(result.is_err());
    }
    #[test]
    fn portpool_allocate_3() {     // Duplicate allocation for a userfails.
        let mut pool = PortPool::new(1000, 2);               // won't run out.
        pool.allocate("SomeService", "fox").unwrap();        // must work. 
        let result = pool.allocate("SomeService", "fox");    // must fail.
        assert!(result.is_err());
    }
    #[test]
    fn portpool_allocate_4() {    // Duplicates allowed if different user:

        let mut pool = PortPool::new(1000, 2);               // won't run out.
        pool.allocate("SomeService", "fox").unwrap();        // must work. 
        let result = pool.allocate("SomeService", "cerizza");   // Should work.
        assert!(result.is_ok());
    }
    #[test] 
    fn portpool_allocate_5()  {  // won't reallocate same port:
        let mut pool = PortPool::new(1000, 2);
        let port1 = pool.allocate("Service_1", "fox").unwrap();
        let port2 = pool.allocate("Service_2", "fox").unwrap();

        // THe ports allocated must be different:

        assert_ne!(port1.port_number, port2.port_number);
    }
    #[test]
    fn usage_1() {
        let pool = PortPool::new(1000, 2);
        assert_eq!(0, pool.usage().len());

    }
    #[test]
    fn usage_2() {                // Single usage:
        let mut pool = PortPool::new(1000, 2);
        let port1 = pool.allocate("Serice_1", "fox").unwrap();
        let usage = pool.usage();
        assert_eq!(1, usage.len());
        assert_eq!(port1.port_number, usage[0].port_number);
        assert_eq!(port1.port_service, usage[0].port_service);
        assert_eq!(port1.port_user, usage[0].port_user);
    }
    #[test]
    fn usage_3() {             // Use a couple of ports:
        let mut pool = PortPool::new(1000, 2);
        let port1 = pool.allocate("service 1", "fox").unwrap();
        let port2 = pool.allocate("service_1", "cerizza").unwrap();
        let mut allocated = vec![port1, port2];
        allocated.sort_by_key(|v| {v.port_number});
        let used = pool.usage();
        assert_eq!(allocated.len(), used.len());
        for i in 0..used.len() {
            assert_eq!(allocated[i].port_number, used[i].port_number);
            assert_eq!(allocated[i].port_service, used[i].port_service);
            assert_eq!(allocated[i].port_user, used[i].port_user);
        }

    }

}
