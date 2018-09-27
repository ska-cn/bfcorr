extern crate bfcorr;
extern crate crossbeam_channel;
extern crate pcap;
extern crate chrono;
extern crate astroalgo;

use astroalgo::sidereal::IntoApparentGreenSidereal;
use bfcorr::calc_corr_par;
use bfcorr::run_daq;
use std::env;
use std::fs::File;
use std::io::prelude::*;

use crossbeam_channel as channel;
use pcap::Capture;
fn main() {
    let args: Vec<_> = env::args().collect();
    let interface1 = args[1].to_string();
    let interface2 = args[2].to_string();
    let ch1 = 400;
    let ch2 = 1640;
    let nch = ch2 - ch1;
    let recv1 = run_daq(&interface1, 60000, nch, 320000, 16);
    let recv2 = run_daq(&interface2, 60000, nch, 320000, 16);
    //let recv=run_daq("ens5f1", 60000, nch, 80000, 16);

    let mut current_chunk_id = 0;

    loop {
        println!("Q len={} {}", recv1.len(), recv2.len());
        let (chunk_id1, data1) = recv1.recv().unwrap();
        let (chunk_id2, data2) = recv2.recv().unwrap();
        println!("{} {}", chunk_id1, chunk_id2);
        let (xx, xy, yy) = if chunk_id1 == chunk_id2 {
            (
                calc_corr_par(&data1, &data1, ch2 - ch1),
                calc_corr_par(&data1, &data2, ch2 - ch1),
                calc_corr_par(&data2, &data2, ch2 - ch1),
            )
        } else if chunk_id1 < chunk_id2 {
            let (chunk_id1, data1) = recv1.recv().unwrap();
            assert!(chunk_id1 == chunk_id2);
            (
                calc_corr_par(&data1, &data1, ch2 - ch1),
                calc_corr_par(&data1, &data2, ch2 - ch1),
                calc_corr_par(&data2, &data2, ch2 - ch1),
            )
        } else if chunk_id1 > chunk_id2 {
            let (chunk_id2, data2) = recv2.recv().unwrap();
            assert!(chunk_id1 == chunk_id2);
            (
                calc_corr_par(&data1, &data1, ch2 - ch1),
                calc_corr_par(&data1, &data2, ch2 - ch1),
                calc_corr_par(&data2, &data2, ch2 - ch1),
            )
        } else {
            panic!();
        };

        let sid=chrono::offset::Utc::now().naive_utc().apparent_green_sidereal_angle().0;
        println!("{} {}", chunk_id1, chunk_id2);

        let mut file_xx = File::create("spec_xx.txt").unwrap();
        let mut file_xy = File::create("spec_xy.txt").unwrap();
        let mut file_yy = File::create("spec_yy.txt").unwrap();
        for i in 0..yy.len() {
            writeln!(
                &mut file_xx,
                "{} {}",
                (i + ch1) as f64 / 2048.0 * 250.0,
                xx[i].re
            );
            writeln!(
                &mut file_xy,
                "{} {} {}",
                (i + ch1) as f64 / 2048.0 * 250.0,
                xy[i].re,xy[i].im
            );
            writeln!(
                &mut file_yy,
                "{} {}",
                (i + ch1) as f64 / 2048.0 * 250.0,
                yy[i].re
            );
        }

        let mut bin_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("xx.bin")
            .expect("cannot open file");
        let data=unsafe{std::slice::from_raw_parts(xx.as_ptr() as *const u8, xx.len()*8*2)};
        bin_file.write(data);

        let mut bin_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("yy.bin")
            .expect("cannot open file");
        let data=unsafe{std::slice::from_raw_parts(yy.as_ptr() as *const u8, xx.len()*8*2)};
        bin_file.write(data);

        let mut bin_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("xy.bin")
            .expect("cannot open file");
        let data=unsafe{std::slice::from_raw_parts(xy.as_ptr() as *const u8, xx.len()*8*2)};
        bin_file.write(data);
        let mut sidereal_file=std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("sidereal.txt")
            .expect("cannot open file");

        writeln!(sidereal_file, "{}",sid);

    }
    //assert!(false);
}
