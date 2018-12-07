extern crate bfcorr;
extern crate crossbeam_channel;
extern crate pcap;

use bfcorr::calc_corr_par;
use bfcorr::run_daq;
use std::env;
use std::fs::File;
use std::io::prelude::*;

use crossbeam_channel as channel;
use pcap::Capture;
fn main() {
    let args: Vec<_> = env::args().collect();
    let interface = args[1].to_string();
    let ch1 = 400;
    let ch2 = 1640;
    let nch = ch2 - ch1;
    let recv = run_daq(&interface, 60000, nch, 80000, 4);
    //let recv=run_daq("ens5f1", 60000, nch, 80000, 16);

    while let Ok((chunk_id, data)) = recv.recv() {
        //println!("a");

        let spec = calc_corr_par(&data, &data, ch2 - ch1);
        let spec = calc_corr_par(&data, &data, ch2 - ch1);
        let spec = calc_corr_par(&data, &data, ch2 - ch1);
        //println!("b");
        //println!("{} {} {} ",chunk_id,  recv.len(),spec.len());

        println!("chunk_id={} Q len={}", chunk_id, recv.len());
        let mut file = File::create("spec.txt").unwrap();
        for i in 0..spec.len() {
            writeln!(
                &mut file,
                "{} {}",
                (i + ch1) as f64 / 2048.0 * 250.0,
                spec[i].re
            );
        }
        //break;
    }

    assert!(false);
}
