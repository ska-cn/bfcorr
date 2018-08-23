use std::net::UdpSocket;


fn main() {
    let socket = UdpSocket::bind("0.0.0.0:6000").unwrap();
    socket.set_nonblocking(false).unwrap();

    let mut buf = vec![0_u8;65536];
    let (num_bytes, src_addr)=socket.recv_from(&mut buf).unwrap();
    println!("{}", num_bytes);
}

