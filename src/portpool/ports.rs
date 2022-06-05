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
    pub fn new(start : u16, n : u16) -> PortPool {
        // Generate the unused port pool

        let mut unused = HashSet::new();
        for p in start..(start+n - 1) {
            unused.insert(p);
        }
        PortPool {
            used : HashMap::new(),
            unused : unused
        }
    }
    fn mark_used(&mut self, port: u16) {
        self.unused.remove(&port);
    }
    fn get_unused(&self) -> u16 {
        let pport = self.unused.iter().next().expect("Bug non-empty free port pool iterator failed");
        *pport
    }
    pub fn allocate(&mut self, service : &str, user: &str) -> Result<UsedPort, String> {
        if self.unused.len() == 0 {
            return Err(String::from("No free ports available"))
        } else {

            let  port = self.get_unused();
            
            self.mark_used(port);
            self.used.insert(port, UsedPort::new(port, service, user));
            Ok(UsedPort::new(port, service, user))
        }
    }
    pub fn usage(&self) -> Vec<UsedPort> {
        let mut result : Vec<UsedPort> = Vec::new();
        for (_, value) in &self.used {
            let u = UsedPort::new(
                value.port(), value.service().as_str(), value.user().as_str()
            );
            result.push(u);
        }
        result
    }
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

// PortPool type:
}
