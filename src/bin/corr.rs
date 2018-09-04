extern crate pcap;
extern crate crossbeam_channel;
extern crate bfcorr;


use std::fs::File;
use std::io::prelude::*;
use std::env;
use bfcorr::run_daq;
use bfcorr::calc_corr_par;

use pcap::Capture;
use crossbeam_channel as channel;
fn main(){
    
    let args:Vec<_>=env::args().collect();
    let interface1=args[1].to_string();
    let interface2=args[2].to_string();
    let ch1=1000;
    let ch2=1032;
    let nch=ch2-ch1;
    let recv1=run_daq(&interface1, 60000, nch, 80000, 16);
    let recv2=run_daq(&interface2, 60000, nch, 80000, 16);
    //let recv=run_daq("ens5f1", 60000, nch, 80000, 16);

    let mut current_chunk_id=0;
    
    loop{
        println!("Q len={} {}", recv1.len(), recv2.len());
        let (chunk_id1, data1)=recv1.recv().unwrap();
        let (chunk_id2, data2)=recv2.recv().unwrap();
        let (xx, xy, yy)=
        if chunk_id1==chunk_id2{
            (calc_corr_par(&data1, &data1, ch2-ch1),
            calc_corr_par(&data1, &data2, ch2-ch1),
            calc_corr_par(&data2, &data2, ch2-ch1))

        }
        else if chunk_id1<chunk_id2{
            let (chunk_id1, data1)=recv1.recv().unwrap();
            assert!(chunk_id1==chunk_id2);
            (calc_corr_par(&data1, &data1, ch2-ch1),
            calc_corr_par(&data1, &data2, ch2-ch1),
            calc_corr_par(&data2, &data2, ch2-ch1))
        }
        else if chunk_id1>chunk_id2{
            let (chunk_id2, data2)=recv2.recv().unwrap();
            assert!(chunk_id1==chunk_id2);
            (calc_corr_par(&data1, &data1, ch2-ch1),
            calc_corr_par(&data1, &data2, ch2-ch1),
            calc_corr_par(&data2, &data2, ch2-ch1))
        }else{
            panic!();
        };
        println!("{} {}", chunk_id1, chunk_id2);

        let mut file=File::create("spec.txt").unwrap();
        for i in 0..yy.len(){
           writeln!(&mut file, "{} {}", (i+ch1) as f64/2048.0*250.0, yy[i].re);
        }
    }    
    //assert!(false);
}
