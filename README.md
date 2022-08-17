### portman

This is a Rust replacement for the NSCLDAQ port manager.  It manages a pool of ports that can be
allocated to servers.  The servers are then published and discoverable through the port manager.
This should be run on tightly controlled networks.

Run as:

 portman   --listen-port=p1 --port-base=p2 --num-ports=n


Where

*   --listen-port specifies the port on which the portman server will listen for connections.
*   --port-base specifies the base of the set of ports managed by the server.
*   --num-ports specifies the number of ports managed by the port manager.
