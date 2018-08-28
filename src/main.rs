extern crate num_complex;
extern crate num_traits;

use std::net::UdpSocket;
use num_complex::Complex;


fn calc_spec(data:&Vec<Complex<i16> >)->Vec<f64>{
    let nch=2048-512;
    let mut cnt=0;
    let mut spec=vec![0.0;nch];
    println!("{}", data.len()/nch);
    for d in data.chunks(nch){        
        for j in 0..nch{
            //println!("{}", j);
            spec[j]+=d[j].norm_sqr() as f64;
        }
    }
    spec
}

fn calc_corr(data1:&Vec<Complex<i16>>, data2:&Vec<Complex<i16>>, nch:usize)->Vec<Complex<f64> >{
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

fn main() {
    let nch=2048-512;
    let nchunk=80000;
    let buf_size=nchunk*nch;

    let buff= 
    {
        let mut temp_buf=vec![1_i16; buf_size*2];
        let ptr=temp_buf.as_mut_ptr();
        std::mem::forget(temp_buf);
        unsafe{Vec::from_raw_parts(ptr as *mut Complex<i16>, buf_size, buf_size)}
    };
    
    //calc_corr(&buff, &buff, nch);
}

