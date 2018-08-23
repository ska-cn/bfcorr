use std::net::UdpSocket;


fn main() {
    let socket = UdpSocket::bind("0.0.0.0:60000").unwrap();
    socket.set_nonblocking(false).unwrap();

    let mut buf = vec![0_u8;65536*1024];
    let mut shift=0_usize;
    for i in 0..128{
        let (num_bytes, _src_addr) = socket.recv_from(&mut buf[shift..]).unwrap();

        let header: &[u64] = unsafe { std::mem::transmute(&buf[num_bytes..(num_bytes+8)]) };

        shift+=num_bytes;
        println!("{}", header[0]);
    }
}

