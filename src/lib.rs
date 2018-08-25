extern crate pcap;

extern crate crossbeam_channel;
use crossbeam_channel as channel;
use pcap::Capture;

pub const ID_MASK: u64 = ((1_u64 << 42) - 1);
pub const BYTES_PER_NUMBER: usize = 2;

pub fn run_daq(
    dev_name: &str,
    port: u16,
    nchannels: usize,
    nchunks: usize,
    queue_depth: usize,
) -> channel::Receiver<(usize, Vec<i16>)> {
    let dev = pcap::Device {
        name: dev_name.to_string(),
        desc: None,
    };

    let mut cap = Capture::from_device(dev)
        .unwrap()
        .timeout(1000000000)
        .buffer_size(512 * 1024 * 1024)
        .open()
        .unwrap();
    cap.filter(&format!("dst port {}", port)).unwrap();
    //cap.next().unwrap();
    let buf_size = nchannels * nchunks * 2;

    let (send, recv): (channel::Sender<(usize, Vec<i16>)>, channel::Receiver<(usize, Vec<i16>)>) =
        channel::bounded(queue_depth);

    let recv1=recv.clone();
    let th = std::thread::spawn(move || {
        let mut old_id={
            let packet=cap.next().unwrap();
            let data=&packet.data[42..];
            let payload=&data[8..];
            let header:&[u64]=unsafe { std::mem::transmute(&data[0..(0 + 8)]) };
            let id=header[0] & ID_MASK;
            id
        };
        let mut cnt=0;
        let mut buf = vec![0_i16; buf_size];
        while let Ok(packet) = cap.next() {
            //println!("received packet! {:?}", packet);
            //println!("{}", packet.data.len());
            let data: &[u8] = &packet.data[42..];
            //println!("{}", data.len());
            let payload = &data[8..];
            if payload.len() != nchannels * BYTES_PER_NUMBER * 2 {
                println!("{}", payload.len());
            }
            assert!(payload.len() == nchannels * BYTES_PER_NUMBER * 2);
            let header: &[u64] = unsafe { std::mem::transmute(&data[0..(0 + 8)]) };
            //let id=header[0] & 0b000000000000000000000111111111111111111111111111111111111111111_u64;
            let id = header[0] & ID_MASK;

            if (id as usize/nchunks)>(old_id as usize/nchunks){
                let old_buf=std::mem::replace(&mut buf, vec![0_i16; buf_size]);
                if recv1.is_full(){
                    recv1.recv();
                }
                send.send((old_id as usize/nchunks, old_buf));
                println!("a");
            }

            let trunk_id = (id as usize) % nchunks;
            let converted_data: &[i16] = unsafe {
                std::slice::from_raw_parts(payload.as_ptr() as *const i16, nchannels * 2)
            };
            //println!("{} {} {}", payload.len(), converted_data.len(), nchannels*2);
            buf[trunk_id as usize * nchannels * 2..((trunk_id + 1) as usize * nchannels * 2)]
                .copy_from_slice(converted_data);

            old_id=id;
            if cnt%10000==0{
                println!("{}", id);
            }
            cnt+=1;
        }
    });
    recv
}
