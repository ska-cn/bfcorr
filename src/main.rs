use std::net::UdpSocket;


fn main() {
    let socket = UdpSocket::bind("0.0.0.0:60000").unwrap();
    socket.set_nonblocking(false).unwrap();

    let mut buf = vec![0_u8;65536*1024];
    let mut shift=0_usize;
    let mut package_size=0;
    for _i in 0..1024{
        let (num_bytes, _src_addr) = socket.recv_from(&mut buf[shift..]).unwrap();

        let header: &[u64] = unsafe { std::mem::transmute(&buf[shift..(shift+8)])} ;
        let id=header[0] & 0b000000000000000000000111111111111111111111111111111111111111111_u64;
        //let id=header[0];
        shift+=num_bytes;
        //println!("{}", id);
        package_size=num_bytes;
    }
    for i in 0..1024{
        let shift=i*package_size;
        let header:&[u64]=unsafe{std::mem::transmute(&buf[shift..(shift+8)])};
        let id=header[0] & 0b000000000000000000000111111111111111111111111111111111111111111_u64;
        println!("{}", id);
    }
}

