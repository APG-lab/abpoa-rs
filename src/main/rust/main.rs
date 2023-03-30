
use ::abpoa::abpoa;
use ::abpoa::helper;
use log::debug;


pub fn check_one ()
{
        let ref_name = String::from ("chr1");
        let ref_seq = String::from ("AAAACATAAAA");

        let sequences = [
            "CAT",
            "CAT"
            ].iter ()
            .map (|s| s.to_string ())
            .collect::<Vec<String>> ();

        let res: Result<(), helper::PublicError> = (|| {
            let sequence_graph_bytes = ::abpoa::reference_as_sequence_graph (ref_name, ref_seq)?;
            let output = abpoa::fetch_sequence_graph_existing (sequence_graph_bytes, sequences, true, true)?;
            debug! ("output: {:?}", output);
            Ok (())
        })();
        match res
        {
            Ok (_) => {},
            Err (e) => {
                println! ("res: {:?}", e);
            }
        }
}

fn main () {
    env_logger::init ();

    debug! ("Check library");
    check_one ();

}

