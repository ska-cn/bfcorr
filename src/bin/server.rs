extern crate pcap;
extern crate crossbeam_channel;
extern crate bfcorr;

use bfcorr::run_daq;
use pcap::Capture;
use crossbeam_channel as channel;
fn main(){

    let nch=1536;
    let recv=run_daq("eth3", 60000, nch, 80000, 4);


    while let Some((chunk_id, data))=recv.recv(){
        println!("b");
        //let x=data.iter().fold(0.0, |x, &y|{x+(y*y.conj()).re as f64});
        let mut spec=vec![0.0;nch];
        for s in data.chunks(nch){
            for i in 0..nch{
                spec[i]+=s[i].norm_sqr() as f64;
            }
        }

        println!("{} {} {} {}",chunk_id,  recv.len(),data.len(), spec.len());
        //println!("{}", data.len());
    }

    assert!(false);
}
