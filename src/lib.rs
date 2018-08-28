extern crate pcap;
extern crate num_complex;
extern crate num_traits;
extern crate crossbeam_channel;
use crossbeam_channel as channel;
use pcap::Capture;
use num_complex::Complex;
use num_traits::identities::Zero;
pub const ID_MASK: u64 = ((1_u64 << 42) - 1);
pub const BYTES_PER_NUMBER: usize = 2;

pub fn run_daq(
    dev_name: &str,
    port: u16,
    nchannels: usize,
    nchunks: usize,
    queue_depth: usize,
) -> channel::Receiver<(usize, Vec<Complex<i16>>)> {
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
    let buf_size = nchannels * nchunks;

    let (send, recv): (channel::Sender<(usize, Vec<Complex<i16>>)>, channel::Receiver<(usize, Vec<Complex<i16>>)>) =
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
        let mut buf:Vec<Complex<i16>> = vec![Complex::new(0,0); buf_size];
        while let Ok(packet) = cap.next() {
            //println!("received packet! {:?}", packet);
            //println!("{}", packet.data.len());
            let data: &[u8] = &packet.data[42..];
            //println!("{}", data.len());
            let payload = &data[8..];
            if payload.len() != nchannels * BYTES_PER_NUMBER * 2 {
                println!("len={}", payload.len());
            }
            assert!(payload.len() == nchannels * BYTES_PER_NUMBER * 2);
            let header: &[u64] = unsafe { std::mem::transmute(&data[0..(0 + 8)]) };
            //let id=header[0] & 0b000000000000000000000111111111111111111111111111111111111111111_u64;
            let id = header[0] & ID_MASK;

            if (id as usize/nchunks)>(old_id as usize/nchunks){
                let mut new_buf=vec![0_i16; buf_size*2];
                //let old_buf=std::mem::replace(&mut buf, vec![Complex::new(0,0); buf_size]);
                let ptr=new_buf.as_mut_ptr();
                std::mem::forget(new_buf);
                let old_buf=std::mem::replace(&mut buf, unsafe{Vec::from_raw_parts(ptr as *mut Complex<i16>, buf_size, buf_size)});
                if recv1.is_full(){
                    recv1.recv();
                }
                send.send((old_id as usize/nchunks, old_buf));
                println!("a");
            }

            let trunk_id = (id as usize) % nchunks;
            let converted_data: &[Complex<i16>] = unsafe {
                std::slice::from_raw_parts(payload.as_ptr() as *const Complex<i16>, nchannels)
            };
            //println!("{} {} {}", payload.len(), converted_data.len(), nchannels*2);
            buf[trunk_id as usize * nchannels ..((trunk_id + 1) as usize * nchannels)]
                .copy_from_slice(converted_data);

            old_id=id;
            if cnt%100000==0{
                println!("{}", id);
            }
            cnt+=1;
        }
    });
    recv
}


pub fn calc_corr(data1:&Vec<Complex<i16>>, data2:&Vec<Complex<i16>>, nch:usize)->Vec<Complex<f64> >{
    let mut tnt=0;
    let zeros= {
        let mut temp_buf=vec![0_f64; nch*2];
        let ptr=temp_buf.as_mut_ptr();
        std::mem::forget(temp_buf);
        unsafe{Vec::from_raw_parts(ptr as *mut Complex<f64>, nch, nch)}
    };

    data1.chunks(nch).zip(data2.chunks(nch)).fold(zeros, |x, (a, b)|{
      x.iter().zip(a.iter().zip(b.iter())).map(|(x, (&y,&z))|{
          let r=(y*z.conj());
          x+Complex::<f64>::new(r.re as f64, r.im as f64)
          }).collect()
    })
}


pub fn calc_corr1(data1:&Vec<Complex<i16>>, data2:&Vec<Complex<i16>>, nch:usize)->Vec<Complex<f64> >{
    let mut tnt=0;
    let zeros= {
        let mut temp_buf=vec![0_f64; nch*2];
        let ptr=temp_buf.as_mut_ptr();
        std::mem::forget(temp_buf);
        unsafe{Vec::from_raw_parts(ptr as *mut Complex<f64>, nch, nch)}
    };

    zeros
}
