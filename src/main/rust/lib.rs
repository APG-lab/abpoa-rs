

use log::debug;
use std::io::Write;

pub mod abpoa;
mod file;
pub mod helper;


pub fn reference_as_sequence_graph (name: String, sequence: String)
    -> Result<Vec<u8>,helper::PublicError>
{
    let mut buffer = Vec::<u8>::new ();

    buffer.write_all (b"H\tReference graph")?;
    buffer.write_all (format! ("S\t{}\t{}", name, sequence).as_bytes ())?;
    Ok (buffer)
}

pub fn fetch_sequence_graph (sequences: Vec<String>)
{
    debug! ("fetch_sequence_graph");
}

#[cfg(test)]
mod tests
{
    use crate::abpoa;
    use crate::helper;
    use log::debug;
    use std::sync;

    static INIT: sync::Once = sync::Once::new ();

    fn init ()
    {
        INIT.call_once (env_logger::init);
    }

    #[test]
    fn test_full_width ()
    {
        init ();
        debug! ("test full width");
        let ref_name = String::from ("chr1");
        let ref_seq = String::from ("AAAACATAAAA");

        let sequences = [
            "CAT"
            ].iter ()
            .map (|s| s.to_string ())
            .collect::<Vec<String>> ();

        let res: Result<(), helper::PublicError> = (|| {
            let sequence_graph_bytes = super::reference_as_sequence_graph (ref_name, ref_seq)?;
            abpoa::fetch_sequence_graph_existing (sequence_graph_bytes, sequences);
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
}
