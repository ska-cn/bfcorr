use std::net::UdpSocket;

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:7000").unwrap();
    let mut buf = [0; 1024];
    buf[1] = 1;
    socket.send_to(&buf, "127.0.0.1:60000").unwrap();
}
