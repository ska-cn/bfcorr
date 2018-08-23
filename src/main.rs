use std::net::UdpSocket;


fn main() {
    let socket = UdpSocket::bind("0.0.0.0:60000").unwrap();
    socket.set_nonblocking(false).unwrap();
    let niter=2048;
    let mut buf = vec![0_u8;16384*niter];
    let mut shift=0_usize;
    let mut package_size=0;

    for _i in 0..niter{
        let (num_bytes, _src_addr) = socket.recv_from(&mut buf[shift..]).unwrap();

        let header: &[u64] = unsafe { std::mem::transmute(&buf[shift..(shift+8)])} ;
        let id=header[0] & 0b000000000000000000000111111111111111111111111111111111111111111_u64;
        //let id=header[0];
        shift+=num_bytes;
        //println!("{}", id);
        package_size=num_bytes;
    }
    let mut id0=0;
    let mut id1=0;
    for i in 0..niter{
        let shift=i*package_size;
        let header:&[u64]=unsafe{std::mem::transmute(&buf[shift..(shift+8)])};
        let id=header[0] & 0b000000000000000000000111111111111111111111111111111111111111111_u64;
        println!("{}", id);
        if i==0{
            id0=id;
        }
        if i==niter-1{
            id1=id;
        }
    }
    println!("{}", id1-id0);
}

