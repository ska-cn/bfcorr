extern crate endian_trait;
extern crate num_complex;
extern crate num_traits;

fn main() {
    let a: i16 = 3;
    println!("{} {}", a, i16::from_be(a));
}
