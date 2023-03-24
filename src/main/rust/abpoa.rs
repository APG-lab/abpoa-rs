
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

include! (concat! (env! ("OUT_DIR"), "/bindings.rs"));

//use crate::file;
use crate::helper;
use libc;
use log::debug;
use std::ffi;
use std::io;
use std::mem;
use std::os;
use std::os::fd::AsRawFd;
use std::ptr;

pub fn sequence_encode_box_slice (sequence: String)
    -> Box<[u8]>
{
    sequence.chars ()
        .into_iter ()
        .map (|c| {
            match c
            {
                'a' => 0,
                'c' => 1,
                'g' => 2,
                't' => 3,
                'n' => 4,
                'A'=> 0,
                'C' => 1,
                'G' => 2,
                'T' => 3,
                'N' => 4,
                _ => 5
            }
        })
        .collect::<Vec<u8>> ().into_boxed_slice ()
}

pub fn fetch_sequence_graph_existing (sequence_graph: Vec<u8>, sequences: Vec<String>)
    -> Result<(), helper::PublicError>
{
    let chunk_read_n: os::raw::c_int = 1024;
    let n_seqs = sequences.len () as os::raw::c_int;

    unsafe {
        
        let ab = abpoa_init ();
        let abpt = abpoa_init_para ();

        let sg = ffi::CString::new (sequence_graph)?;

        // output options
        (*abpt).set_out_msa (1); // generate Row-Column multiple sequence alignment(RC-MSA), set 0 to disable
        (*abpt).set_out_cons (1); // generate consensus sequence, set 0 to disable
        (*abpt).w = 6;
        (*abpt).k = 9;
        (*abpt).min_w = 10; // minimizer-based seeding and partition
        (*abpt).set_progressive_poa (1);
        (*abpt).incr_fn = libc::strdup (sg.as_ptr ());

        abpoa_post_set_para (abpt);

        let n_seqs = sequences.len () as os::raw::c_int;

        // convert sequences to c type
        let mut seq_lens_slice = sequences.iter ().map (|x| x.chars ().count () as os::raw::c_int).collect::<Vec<_>> ().into_boxed_slice ();
        let seq_lens = seq_lens_slice.as_mut_ptr ();

        let sequences_encoded_boxed_sliced = sequences.iter ().cloned ().map (sequence_encode_box_slice).collect::<Vec<_>> ();
        let sequences_slice = sequences_encoded_boxed_sliced.into_iter ()
            .map (|x| {
                Box::into_raw (x) as *mut u8
            })
            .collect::<Vec<_>> ()
            .into_boxed_slice ();

        let bseqs = Box::into_raw (sequences_slice) as *mut *mut u8;

        let seq_names: *mut *mut::std::os::raw::c_char = ptr::null_mut ();
        let weights: *mut *mut i32 = ptr::null_mut ();
        // perform abpoa-msa
        abpoa_msa(ab, abpt, n_seqs, seq_names, seq_lens, bseqs, weights, stdout);
    }
    Ok (())
}

