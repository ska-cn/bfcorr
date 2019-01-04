#![allow(clippy::type_complexity)]
extern crate crossbeam_channel;
extern crate endian_trait;
extern crate num_complex;
extern crate num_traits;
extern crate pnet;
extern crate rayon;

use rayon::prelude::*;

use crossbeam_channel as channel;
use num_complex::Complex;
use pnet::datalink::interfaces;
use pnet::datalink::{channel, Config, ChannelType, Channel};
//use pcap::Capture;
pub const ID_MASK: u64 = ((1_u64 << 42) - 1);
pub const BYTES_PER_NUMBER: usize = 2;

pub fn run_daq(
    dev_name: &str,
    port: u16,
    nchannels: usize,
    nchunks: usize,
    queue_depth: usize,
) -> channel::Receiver<(usize, Vec<Complex<i16>>)> {

    let dev=interfaces().into_iter().filter(|x|{x.name==dev_name}).nth(0).expect("Cannot find dev");


    let cfg=Config{write_buffer_size:1024, read_buffer_size:((1_usize << 31) - 1), read_timeout:None, write_timeout:None, channel_type:ChannelType::Layer2, bpf_fd_attempts:1000,
        };

    let mut cap=
    if let Channel::Ethernet(_, cap)=channel(&dev, cfg).expect("canot open channel"){
        cap
    }else{
        panic!();
    };

    let packet_len = nchannels * BYTES_PER_NUMBER * 2 + 50;
    /*cap.filter(&format!(
        "less {} and greater {} and dst port {}",
        packet_len, packet_len, port
    )).unwrap();*/
    //cap.next().unwrap();
    let buf_size = nchannels * nchunks;

    let (send, recv): (
        channel::Sender<(usize, Vec<Complex<i16>>)>,
        channel::Receiver<(usize, Vec<Complex<i16>>)>,
    ) = channel::bounded(queue_depth);

    let recv1 = recv.clone();
    let _th = std::thread::spawn(move || {
        let mut old_id = loop {
            let packet = cap.next().unwrap();
            if packet.len()!=packet_len{
                continue
            }
            let data = &packet[42..];
            //let payload = &data[8..];
            //let header: &[u64] = unsafe { std::mem::transmute(&data[0..(0 + 8)]) };
            let mut header = 0_u64;
            unsafe { std::slice::from_raw_parts_mut(&mut header as *mut u64 as *mut u8, 8) }
                .copy_from_slice(&data[0..8]);

            /*
            let header: &[u64] = unsafe {&*(&data[0..8] as *const [u8] as *const [u64]) };
            assert!(payload.len() == nchannels * BYTES_PER_NUMBER * 2);
            header[0] & ID_MASK
            */
            break header & ID_MASK
        };
        let mut cnt = 0;
        let mut buf: Vec<Complex<i16>> = vec![Complex::new(0, 0); buf_size];
        let mut id_while_ago = 0;
        while let Ok(packet) = cap.next() {
            //println!("received packet! {:?}", packet);
            //println!("{}", packet.data.len());
            //continue;
            let data: &[u8] = &packet[42..];
            //println!("{}", data.len());
            let payload = &data[8..];
            //println!("{} {}", data.len(), payload.len());

            assert!(payload.len() == nchannels * BYTES_PER_NUMBER * 2);
            //let header: &[u64] = unsafe { std::mem::transmute(&data[0..(0 + 8)]) };
            //let header: &[u64] = unsafe { &*(&data[0..8] as *const [u8] as *const [u64])};
            //let id=header[0] & 0b000000000000000000000111111111111111111111111111111111111111111_u64;
            //println!("header={}", header[0]);
            let mut header = 0_u64;
            unsafe { std::slice::from_raw_parts_mut(&mut header as *mut u64 as *mut u8, 8) }
                .copy_from_slice(&data[0..8]);

            /*
            let header: &[u64] = unsafe {&*(&data[0..8] as *const [u8] as *const [u64]) };
            assert!(payload.len() == nchannels * BYTES_PER_NUMBER * 2);
            header[0] & ID_MASK
            */
            let id = header & ID_MASK;

            if (id as usize / nchunks) > (old_id as usize / nchunks) {
                let mut new_buf = vec![0_i16; buf_size * 2];
                //let old_buf=std::mem::replace(&mut buf, vec![Complex::new(0,0); buf_size]);
                let ptr = new_buf.as_mut_ptr();
                std::mem::forget(new_buf);
                let old_buf = std::mem::replace(&mut buf, unsafe {
                    Vec::from_raw_parts(ptr as *mut Complex<i16>, buf_size, buf_size)
                });
                if recv1.is_full() {
                    recv1.recv().unwrap();
                }
                send.send((old_id as usize / nchunks, old_buf)).unwrap();
                //println!("a");
            }

            let trunk_id = (id as usize) % nchunks;

            /*
            let converted_data: &[Complex<i16>] = unsafe {
                std::slice::from_raw_parts(payload.as_ptr() as *const Complex<i16>, nchannels)
            };
            //println!("{} {} {}", payload.len(), converted_data.len(), nchannels*2);
            buf[trunk_id as usize * nchannels..((trunk_id + 1) as usize * nchannels)]
                .copy_from_slice(converted_data);
            */
            unsafe {
                std::slice::from_raw_parts_mut(
                    buf[trunk_id as usize * nchannels..((trunk_id + 1) as usize * nchannels)]
                        .as_mut_ptr() as *mut u8,
                    nchannels * 4,
                ).copy_from_slice(payload);
            }

            old_id = id;
            if cnt % 100_000 == 0 {
                let lost_ratio = 1.0 - 100_000. / (id as f64 - id_while_ago as f64);
                println!("id={} drop ratio: {:.3}", id, lost_ratio);
                id_while_ago = id;
            }
            cnt += 1;
        }
    });
    recv
}

pub fn calc_corr(data1: &[Complex<i16>], data2: &[Complex<i16>], nch: usize) -> Vec<Complex<f64>> {
    assert!(data1.len() == data2.len());
    let zeros = {
        let mut temp_buf = vec![0_f64; nch * 2];
        let ptr = temp_buf.as_mut_ptr();
        std::mem::forget(temp_buf);
        unsafe { Vec::from_raw_parts(ptr as *mut Complex<f64>, nch, nch) }
    };

    data1
        .chunks(nch)
        .zip(data2.chunks(nch))
        .fold(zeros, |x, (a, b)| {
            x.iter()
                .zip(a.iter().zip(b.iter()))
                .map(|(x, (&y, &z))| {
                    let y1 = Complex::<f64>::new(f64::from(y.re), f64::from(y.im));
                    let z1 = Complex::<f64>::new(f64::from(z.re), f64::from(z.im));
                    let r = y1 * z1.conj();
                    if r.re < 0.0 {
                        println!("{:?} {:?}", y1, z1);
                    }
                    assert!(r.re >= 0.0);
                    x + r
                }).collect()
        })
}

pub fn calc_corr_par(
    data1: &[Complex<i16>],
    data2: &[Complex<i16>],
    nch: usize,
) -> Vec<Complex<f64>> {
    assert!(data1.len() == data2.len());
    let zeros = || {
        let mut temp_buf = vec![0_f64; nch * 2];
        let ptr = temp_buf.as_mut_ptr();
        std::mem::forget(temp_buf);
        unsafe { Vec::from_raw_parts(ptr as *mut Complex<f64>, nch, nch) }
    };

    data1
        .par_iter()
        .zip(data2.par_iter())
        .map(|(&a, &b)| {
            let a1 = Complex::<f64>::new(f64::from(a.re), f64::from(a.im));
            let b1 = Complex::<f64>::new(f64::from(b.re), f64::from(b.im));
            a1 * b1.conj()
        }).chunks(nch)
        .reduce(zeros, |a, b| {
            a.iter().zip(b.iter()).map(|(x, y)| x + y).collect()
        })
}

pub fn calc_corr_coeff_par(
    data1: &[Complex<i16>],
    data2: &[Complex<i16>],
    nch: usize,
) -> Vec<Complex<f64>> {
    assert!(data1.len() == data2.len());
    let zeros = || {
        let mut temp_buf = vec![0_f64; nch * 2];
        let ptr = temp_buf.as_mut_ptr();
        std::mem::forget(temp_buf);
        unsafe { Vec::from_raw_parts(ptr as *mut Complex<f64>, nch, nch) }
    };

    data1
        .par_iter()
        .zip(data2.par_iter())
        .map(|(&a, &b)| {
            let a1 = Complex::<f64>::new(f64::from(a.re), f64::from(a.im));
            let b1 = Complex::<f64>::new(f64::from(b.re), f64::from(b.im));
            let n = a1.norm() * b1.norm();
            if n == 0.0 {
                Complex::<f64>::new(0.0, 0.0)
            } else {
                a1 * b1.conj() / n
            }
        }).chunks(nch)
        .reduce(zeros, |a, b| {
            a.iter().zip(b.iter()).map(|(x, y)| x + y).collect()
        })
}

pub fn calc_mean_par(data1: &[Complex<i16>], nch: usize) -> Vec<Complex<i64>> {
    let zeros = || {
        let mut temp_buf = vec![0_i64; nch * 2];
        let ptr = temp_buf.as_mut_ptr();
        std::mem::forget(temp_buf);
        unsafe { Vec::from_raw_parts(ptr as *mut Complex<i64>, nch, nch) }
    };
    let _nchunk = data1.len() / nch;

    data1
        .par_iter()
        .map(|&a| Complex::<i64>::new(i64::from(a.re), i64::from(a.im)))
        .chunks(nch)
        .reduce(zeros, |a, b| {
            a.iter().zip(b.iter()).map(|(x, y)| x + *y).collect()
        }) //.iter().map(|&x|{x/nchunk as f64}).collect()
}

pub fn calc_mean_par_be(data1: &[Complex<i16>], nch: usize) -> Vec<Complex<i64>> {
    let zeros = || {
        let mut temp_buf = vec![0_i64; nch * 2];
        let ptr = temp_buf.as_mut_ptr();
        std::mem::forget(temp_buf);
        unsafe { Vec::from_raw_parts(ptr as *mut Complex<i64>, nch, nch) }
    };
    let _nchunk = data1.len() / nch;

    data1
        .par_iter()
        .map(|&a| {
            //println!("{} {}", a.re.to_le(), a.re);
            Complex::<i64>::new(i64::from(i16::from_be(a.re)), i64::from(i16::from_be(a.im)))
            //let a1 = Complex::<i64>::new(a.re as i64, a.im as i64);
        }).chunks(nch)
        .reduce(zeros, |a, b| {
            a.iter().zip(b.iter()).map(|(_x, y)| *y).collect()
        }) //.iter().map(|&x|{x/nchunk as f64}).collect()
}

pub fn calc_corr1(
    _data1: &[Complex<i16>],
    _data2: &[Complex<i16>],
    nch: usize,
) -> Vec<Complex<f64>> {
    let _tnt = 0;

    let mut temp_buf = vec![0_f64; nch * 2];
    let ptr = temp_buf.as_mut_ptr();
    std::mem::forget(temp_buf);
    unsafe { Vec::from_raw_parts(ptr as *mut Complex<f64>, nch, nch) }
}
