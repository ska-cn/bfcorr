extern crate pcap;
extern crate crossbeam_channel;
use pcap::Capture;
use crossbeam_channel as channel;
fn main(){
    let mut cnt=0;
    const ID_MASK:u64=((1_u64<<42)-1);
    const BUF_SIZE:usize=512*1024*1024;
    const PAYLOAD_LEN:usize=4096;
    const NUM_TRUNKS:usize=BUF_SIZE/PAYLOAD_LEN;
    
    
    let d=pcap::Device{name:"eth3".to_string(), desc:None};

    let mut cap = Capture::from_device(d).unwrap()
                .timeout(1000000000)
                .buffer_size(512*1024*1024)
                .open().unwrap();            
    cap.filter("dst port 60000");
    //cap.next().unwrap();


    let (send, recv):(channel::Sender<Vec<u8>>,channel::Receiver<Vec<u8>>)  = channel::bounded(4);

    std::thread::spawn(move || {
    // This call blocks the current thread because the channel is full.
    // It will be able to complete only after the first message is received.
    let mut result=vec![0.0;2048];
    while let Some(data)=recv.recv(){
        
        println!("{} {}", recv.len(),data.len());
    }
    });

    loop{
        let mut buf=vec![0_u8;BUF_SIZE];
        while let Ok(packet) = cap.next() {
            //println!("received packet! {:?}", packet);
            //println!("{}", packet.data.len());
            let data=&packet.data[42..];
            //println!("{}", data.len());
            let payload=&data[8..];
            let header:&[u64]=unsafe{std::mem::transmute(&data[0..(0+8)])};
            //let id=header[0] & 0b000000000000000000000111111111111111111111111111111111111111111_u64;
            let id=header[0] & ID_MASK;
            let trunk_id=(id as usize)%NUM_TRUNKS;
            buf[trunk_id as usize*PAYLOAD_LEN..(trunk_id as usize*PAYLOAD_LEN+PAYLOAD_LEN)].copy_from_slice(payload);

            if cnt%10000==0{
                println!("{} {} {} {}", id, payload.len(), trunk_id, NUM_TRUNKS);
            }
            cnt+=1;
            if trunk_id==NUM_TRUNKS-1{
                break;
            }
        }
        send.send(buf);
    }
}
