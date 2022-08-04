///!
///! This program is a replacement for the NSCLDAQ
///! port manager application.  The port manager provides
///! a TCP/IP server that manages a pool of service ports.
///! Locally running pplications can request a port, which is
///! advertised by the server to both local and remote
///! applications.  
///!
///! ### Motivaction:
///!
///!   Why should we re-write a perfectly good existing application
///! in Rust?  There are a few motivations worth listing here:
///!
///! *   As we (FRIB) use NSCLDAQ in a containerized environment,
///!     A start script for our servers is complicated by the need
///!     to run those servers in a containerized environment.
///!     Rust applications, on the other hand, are self contained
///!     executables.  As a result, they can be run
///!     *outside* the container, in the native system, simplifying
///!     our rc script.
///! *   Rust is an interesting language that may merit use elsewhere
///!     in the NSCLDAQ/SpecTcl ecosystem.  This rewrite provides a
///!     useful, self-contained learning project to understand what
///!     it takes to develope a simple, yet useful, production quality
///!     application in Rust.
///!
///! ### Running the server
///!    The server accepts the following options, which contro how it
///!    works (the Rust Clap crate is used to parse the command line
///!    Clap is an acronym for Command Line Argument Parser and is not
///!    to be confused with the STD):
///!
///!    *  --listen_port  - (required) The port on which our server listens for connections.
///!    *  --port_base    - (required) The lowest port number in the allocation pool
///!    *  --port_count   - (required) The number of ports to allocate to the pool.
///!
///!  ### Program structure:
///!
///!    There will be n+2 threads (counting main), where n is the number of
///!    currently allocated ports:
///!
///!     *  The main thread processes paramters and listens for connections
///!        on the listen_port value.  When connections come in they are
///!        decoded and processed by:
///!     *  The service thread maintains the port pool.  It's given requests
///!        for allocations and allocation usage by the main thread via
///!        channels and, in some cases replies to those requests providing
///!        the desired information via a one-time reply channel that's
///!        provided by the request.   See the portman::resonder module for information
///!        about this thread.
///!     *  In order to ensure ports are released, each application requesting a
///!        port must maintain a connection to this server (one connection per port
///!        allocation).  main spins off a thread to monitor this connection and,
///!        when it becomes readable, asks the service thread to drop the allocation.
///!        In this way, even if a service exits abnormally, its port is released.
///!
///! ### Request and replies:
///!
///!     The server accepts several request types.  With the exception of the
///!     request to list the allocations, requests _must_ come from localhost
///!     (127.0.0.1 or its IPV6 equivalent).  The requests are ASCII strings
///!     terminated by a newline.  Replies will be described in the
///!     description of each request, however a common failure reply is of the form:
///! ```
///!        FAIL human readable reason for the failure.
///! ```
///! #### GIMME service-name user-name
///!
///!     Requests a port allocation.  The service-name  and user-name are
///!     used to advertise the service.  The service-name must be unique
///!     for the user.   Note that this differs from the Tcl port manager
///!     which uniquifies the service-name if needed, much to the confusion
///!     of client applications.  This request must come from the
///!     local host. On success, the reply is of the form:
///!
///! ```
///!     OK portnum
///! ```
///!  
///!     Where *portnum* is the port that was allocated to the service.
///!     The service provider must retain an open connection to the
///!     port manager as the port is released when the connection is dropped
///!     (or, for that matter, since additional messages on the socket are
///!     illegal, if the connection becomes readable).
///!
///! #### LIST
///!    
///!     Lists the port usage.  This request cannot fail, unless there's some
///!     internal error.  The reply is  of the form:
///!
///! ```
///!    OK n
///! ```
///!     Where *n* is the number of lines that follow.  Each subsequent line is of
///!     the form:
///! ```
///!    port-number service-name user-name
///! ```
///!     Where port-number is the number of the listen port allocated to the
///!     service-name, user-name pair.
///!     Once the list has been rendered to the client, the connection is closed
///!     by the server.
///!
///! #### TERMINATE
///!     
///!     Requests the system to exit.  No reponse is given.
///!
///!
///!   
fn main() {}
