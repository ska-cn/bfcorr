use std::net::UdpSocket;

fn main(){
    let socket=UdpSocket::bind("127.0.0.1:7000").unwrap();
    socket.send_to(&[0;1024], "127.0.0.1:6000").unwrap();
}
