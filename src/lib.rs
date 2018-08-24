extern crate pcap;

use std::sync::RwLock;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::cell::RefCell;

pub const ID_MASK:u64=((1_u64<<42)-1);
pub const BYTES_PER_NUMBER:usize=2;


#[derive(Clone)]
pub enum PacketPayload{
    Unused(Vec<i16>),
    Used
}

pub struct DataReceiver{
    chunk_per_buffer:usize,
    payload_bytes:usize,//in bytes
    current_writing_buffer:AtomicUsize,
    buffer:[RwLock<Vec<PacketPayload>>;2],
    capture:RefCell<pcap::Capture<pcap::Active>>
}


impl DataReceiver{
    fn new(device_name:&str, port:usize, 
    chunk_per_buffer:usize,nchannels:usize    
    )->DataReceiver{
        let payload_bytes=nchannels*BYTES_PER_NUMBER;
        let buffer=[RwLock::new(vec![PacketPayload::Used; chunk_per_buffer]), RwLock::new(vec![PacketPayload::Used; chunk_per_buffer])];
        let device=pcap::Device{name:device_name.to_string(), desc:None};
        let mut capture=pcap::Capture::from_device(device).unwrap()
                .timeout(1000000000)
                .buffer_size(512*1024*1024)
                .open().unwrap();
        capture.filter(&format!("dst port {}", port));

        DataReceiver{
            chunk_per_buffer:chunk_per_buffer,    
            payload_bytes:payload_bytes,
            current_writing_buffer:AtomicUsize::new(0),
            buffer:buffer,
            capture:RefCell::new(capture)
        }
    }


    fn run(&self)->!{
        self.current_writing_buffer.store(0, Ordering::Relaxed);
        //let w=vec![3,2];
        loop{
            let mut cap=self.capture.borrow_mut();
            let packet=cap.next().unwrap();
            let data=&packet.data[42..];
            //println!("{}", data.len());
            let payload:&[i16]=unsafe{std::mem::transmute(&data[8..])};
            let header:&[u64]=unsafe{std::mem::transmute(&data[0..(0+8)])};
            //let id=header[0] & 0b000000000000000000000111111111111111111111111111111111111111111_u64;
            let id=header[0] & ID_MASK;
            let current_buffer_id=(id as usize/self.chunk_per_buffer)%2;
            let next_buffer_id=((id as usize+1)/self.chunk_per_buffer)%2;
            //w=4;
            //let trunk_id=(id as usize)%NUM_TRUNKS;
            //buf[trunk_id as usize*PAYLOAD_LEN..(trunk_id as usize*PAYLOAD_LEN+PAYLOAD_LEN)].copy_from_slice(payload);
        }
    }
}
