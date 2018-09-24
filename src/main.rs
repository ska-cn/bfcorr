extern crate num_complex;
extern crate num_traits;

use num_complex::Complex;

use std::io::Write;

fn main() {
    let data:Vec<_>=(1..1024).map(|x|{Complex::<f64>::new(0.0, x as f64/2.0)}).collect();

    let mut bin_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("xx.bin")
        .expect("cannot open file");
    let data1=unsafe{std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len()*8*2)};
    bin_file.write(data1);
}
