extern crate num_complex;
extern crate num_traits;
extern crate endian_trait;
use num_complex::Complex;

use std::io::Write;
use endian_trait::Endian;
fn main() {
    let a:i16=3;
    println!("{} {}", a, i16::from_be(a));
}
