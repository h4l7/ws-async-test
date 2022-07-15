tokio-tungstenite dropped connection test

# Usage
Open three terminals
1. In terminal 1, `cargo run -- debug server`
2. In terminal 2, `cargo run -- debug client`
3. Observe monotonic counter being sent over WebSocket
4. In terminal 3, `sudo iptables -A INPUT -p tcp --dport 1337 -j DROP`
5. Observe absense of monotonic counter reaching the server
6. Wait...
7. Note how the server never terminates the client's connection, even if you C-c in terminal 2.