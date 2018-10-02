extern crate crossbeam_channel;
extern crate num_complex;
extern crate num_traits;
extern crate pcap;
extern crate rayon;
extern crate endian_trait;

use rayon::prelude::*;

use endian_trait::Endian;
use crossbeam_channel as channel;
use num_complex::Complex;
use num_traits::identities::Zero;
use pcap::Capture;
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
        .timeout(10000000)
        .buffer_size(((1_u32 << 31) - 1) as i32)
        .open()
        .unwrap();
    let packet_len = nchannels * BYTES_PER_NUMBER * 2 + 50;
    cap.filter(&format!(
        "less {} and greater {} and dst port {}",
        packet_len, packet_len, port
    )).unwrap();
    //cap.next().unwrap();
    let buf_size = nchannels * nchunks;

    let (send, recv): (
        channel::Sender<(usize, Vec<Complex<i16>>)>,
        channel::Receiver<(usize, Vec<Complex<i16>>)>,
    ) = channel::bounded(queue_depth);

    let recv1 = recv.clone();
    let th = std::thread::spawn(move || {
        let mut old_id = {
            let packet = cap.next().unwrap();
            let data = &packet.data[42..];
            let payload = &data[8..];
            let header: &[u64] = unsafe { std::mem::transmute(&data[0..(0 + 8)]) };
            assert!(payload.len() == nchannels * BYTES_PER_NUMBER * 2);
            let id = header[0] & ID_MASK;
            id
        };
        let mut cnt = 0;
        let mut buf: Vec<Complex<i16>> = vec![Complex::new(0, 0); buf_size];
        let mut id_while_ago = 0;
        while let Ok(packet) = cap.next() {
            //println!("received packet! {:?}", packet);
            //println!("{}", packet.data.len());
            //continue;
            let data: &[u8] = &packet.data[42..];
            //println!("{}", data.len());
            let payload = &data[8..];
            //println!("{} {}", data.len(), payload.len());

            assert!(payload.len() == nchannels * BYTES_PER_NUMBER * 2);
            let header: &[u64] = unsafe { std::mem::transmute(&data[0..(0 + 8)]) };
            //let id=header[0] & 0b000000000000000000000111111111111111111111111111111111111111111_u64;
            //println!("header={}", header[0]);
            let id = header[0] & ID_MASK;

            if (id as usize / nchunks) > (old_id as usize / nchunks) {
                let mut new_buf = vec![0_i16; buf_size * 2];
                //let old_buf=std::mem::replace(&mut buf, vec![Complex::new(0,0); buf_size]);
                let ptr = new_buf.as_mut_ptr();
                std::mem::forget(new_buf);
                let old_buf = std::mem::replace(&mut buf, unsafe {
                    Vec::from_raw_parts(ptr as *mut Complex<i16>, buf_size, buf_size)
                });
                if recv1.is_full() {
                    recv1.recv();
                }
                send.send((old_id as usize / nchunks, old_buf));
                //println!("a");
            }

            let trunk_id = (id as usize) % nchunks;
            let converted_data: &[Complex<i16>] = unsafe {
                std::slice::from_raw_parts(payload.as_ptr() as *const Complex<i16>, nchannels)
            };
            //println!("{} {} {}", payload.len(), converted_data.len(), nchannels*2);
            buf[trunk_id as usize * nchannels..((trunk_id + 1) as usize * nchannels)]
                .copy_from_slice(converted_data);

            old_id = id;
            if cnt % 100000 == 0 {
                let lost_ratio = 1.0 - 100000. / (id as f64 - id_while_ago as f64);
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
                    let y1 = Complex::<f64>::new(y.re as f64, y.im as f64);
                    let z1 = Complex::<f64>::new(z.re as f64, z.im as f64);
                    let r = (y1 * z1.conj());
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
            let a1 = Complex::<f64>::new(a.re as f64, a.im as f64);
            let b1 = Complex::<f64>::new(b.re as f64, b.im as f64);
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
            let a1 = Complex::<f64>::new(a.re as f64, a.im as f64);
            let b1 = Complex::<f64>::new(b.re as f64, b.im as f64);
            let n=a1.norm()*b1.norm();
            if n==0.0{
                Complex::<f64>::new(0.0, 0.0)
            }
            else{
                a1 * b1.conj()/n
            }
        }).chunks(nch)
        .reduce(zeros, |a, b| {
            a.iter().zip(b.iter()).map(|(x, y)| x + y).collect()
        })
}


pub fn calc_mean_par(
    data1: &[Complex<i16>],
    nch: usize,
) -> Vec<Complex<i64>> {
    let zeros = || {
        let mut temp_buf = vec![0_i64; nch * 2];
        let ptr = temp_buf.as_mut_ptr();
        std::mem::forget(temp_buf);
        unsafe { Vec::from_raw_parts(ptr as *mut Complex<i64>, nch, nch) }
    };
    let nchunk=data1.len()/nch;

    data1
        .par_iter()
        .map(|&a| {
            let a1 = Complex::<i64>::new(a.re as i64, a.im as i64);
            a1
        }).chunks(nch)
        .reduce(zeros, |a, b| {
            a.iter().zip(b.iter()).map(|(x, y)| x+*y).collect()
        })//.iter().map(|&x|{x/nchunk as f64}).collect()
}

pub fn calc_mean_par_be(
    data1: &[Complex<i16>],
    nch: usize,
) -> Vec<Complex<i64>> {
    let zeros = || {
        let mut temp_buf = vec![0_i64; nch * 2];
        let ptr = temp_buf.as_mut_ptr();
        std::mem::forget(temp_buf);
        unsafe { Vec::from_raw_parts(ptr as *mut Complex<i64>, nch, nch) }
    };
    let nchunk=data1.len()/nch;

    data1
        .par_iter()
        .map(|&a| {
            //println!("{} {}", a.re.to_le(), a.re);
            let a1 = Complex::<i64>::new(i16::from_be(a.re) as i64, i16::from_be(a.im) as i64);
            //let a1 = Complex::<i64>::new(a.re as i64, a.im as i64);
            a1
        }).chunks(nch)
        .reduce(zeros, |a, b| {
            a.iter().zip(b.iter()).map(|(x, y)| *y).collect()
        })//.iter().map(|&x|{x/nchunk as f64}).collect()
}



pub fn calc_corr1(
    data1: &Vec<Complex<i16>>,
    data2: &Vec<Complex<i16>>,
    nch: usize,
) -> Vec<Complex<f64>> {
    let mut tnt = 0;
    let zeros = {
        let mut temp_buf = vec![0_f64; nch * 2];
        let ptr = temp_buf.as_mut_ptr();
        std::mem::forget(temp_buf);
        unsafe { Vec::from_raw_parts(ptr as *mut Complex<f64>, nch, nch) }
    };

    zeros
}
