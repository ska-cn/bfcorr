extern crate pcap;
extern crate crossbeam_channel;
extern crate bfcorr;


use std::fs::File;
use std::io::prelude::*;
use std::env;
use bfcorr::run_daq;
use bfcorr::calc_corr;

use pcap::Capture;
use crossbeam_channel as channel;
fn main(){
    let args:Vec<_>=env::args().collect();
    let interface=args[1].to_string();
    let nch=2048-512;
    let recv=run_daq(&interface, 60000, nch, 80000, 16);
    //let recv=run_daq("ens5f1", 60000, nch, 80000, 16);


    while let Some((chunk_id, data))=recv.recv(){
        //let mut file=File::create("spec.txt").unwrap();
        //println!("a");
        let spec=calc_corr(&data, &data, nch);
        //println!("b");
        println!("{} {} {} ",chunk_id,  recv.len(),spec.len());
        //println!("{}", data.len());
        //for i in 0..spec.len(){
         //   writeln!(&mut file, "{} {}", (i+512) as f64/2048.0*250.0, spec[i].re);
       // }
        //break;

    }

    //assert!(false);
}
