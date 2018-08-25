extern crate pcap;
extern crate crossbeam_channel;
extern crate bfcorr;

use bfcorr::run_daq;
use pcap::Capture;
use crossbeam_channel as channel;
fn main(){

    let recv=run_daq("eth3", 60000, 1536, 160000, 4);

    while let Some((chunk_id, data))=recv.recv(){
        let x=data.iter().fold(0.0, |x, &y|{x+y as f64*y as f64});
        
        println!("{} {} {} {}",chunk_id,  recv.len(),data.len(), x);
        //println!("{}", data.len());
    }

}
