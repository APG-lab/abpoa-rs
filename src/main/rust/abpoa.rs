
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

include! (concat! (env! ("OUT_DIR"), "/bindings.rs"));

use crate::file;
use crate::helper;
use libc;
use log::debug;
use std::ffi;
use std::os;
use std::ptr;
use std::slice;

#[derive(Clone,Debug)]
pub struct Output
{
    pub gfa: Option<String>,
    pub msa: Option<Vec<String>>
}

pub fn u8_to_bool (v: u8)
    -> bool
{
    match v
    {
        0 => false,
        1 => true,
        _ => panic! ("Invalid bool in u8 {}", v),
    }
}

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

pub fn align_to_reference (ab: *mut abpoa_t, abpt: *mut abpoa_para_t, bseqs: *mut *mut u8, seq_lens: *const os::raw::c_int)
{
    unsafe {
        let abs: *const abpoa_seq_t = (*ab).abs;
        let n_seq = (*abs).n_seq;

        let mut res: abpoa_res_t = abpoa_res_t {
            n_cigar: 0,
            m_cigar: 0,
            graph_cigar: ptr::null_mut (),
            node_s: 0,
            node_e: 0,
            query_s: 0,
            query_e: 0,
            n_aln_bases: 0,
            n_matched_bases: 0,
            best_score: 0
        };

        for i in 0..n_seq as usize
        {
            res.graph_cigar = ptr::null_mut ();
            res.n_cigar = 0;
            let mut exc_beg: os::raw::c_int = 0;
            let mut exc_end: os::raw::c_int = 0;
            //let ptr_exc_beg: *mut os::raw::c_int = &exc_beg;
            if i != 0
            {
                abpoa_subgraph_nodes (ab, abpt, 0, 1, &mut exc_beg, &mut exc_end);
            }
            debug! ("i: {}, beg: {}, end: {}", i, exc_beg, exc_end);
            abpoa_align_sequence_to_subgraph (ab, abpt, exc_beg, exc_end, *bseqs.add (i), *seq_lens.add (i), &mut res);
            abpoa_add_subgraph_alignment (ab, abpt, exc_beg, exc_end, *bseqs.add (i), ptr::null_mut (), *seq_lens.add (i), ptr::null_mut (), res, i as os::raw::c_int, n_seq, 0);
            if res.n_cigar > 0
            {
                libc::free (res.graph_cigar as *mut libc::c_void);
            }
        }
    }
}

pub fn render_msa (ab: *mut abpoa_t)
    -> Vec<String>
{
    let mut res = Vec::new ();
    unsafe {
        let abc: *const abpoa_cons_t = (*ab).abc;
        let s_slice = slice::from_raw_parts ((*abc).msa_base, (*abc).n_seq.try_into ().unwrap ());
        debug! ("s_slice: {:?}", s_slice);
        for i in 0..(*abc).n_seq as usize
        {
            debug! ("i: {} msa_len: {}", i, (*abc).msa_len);
            let a_slice = slice::from_raw_parts (&*s_slice[i], (*abc).msa_len.try_into ().unwrap ());
            debug! ("a_slice: {:?}", a_slice);
            let sa = (0..((*abc).msa_len as usize)).map (|j|
            {
                match &a_slice[j]
                {
                    &0 => 'A',
                    &1 => 'C',
                    &2 => 'G',
                    &3 => 'T',
                    &4 => 'N',
                    _ => '?'
                }
            }).collect::<String> ();
            res.push (sa);
        }
    }
    res
}

pub fn fetch_sequence_graph_existing (sequence_graph: Vec<u8>, sequences: Vec<String>, msa: bool, gfa: bool)
    -> Result<Output, helper::PublicError>
{
    let mut res = Output { gfa: None, msa: None };
    unsafe {
        
        let ab = abpoa_init ();
        let abpt = abpoa_init_para ();

        let sg = ffi::CString::new (sequence_graph)?;

        // output options
        (*abpt).set_out_msa (if msa { 1 } else { 0 }); // generate Row-Column multiple sequence alignment(RC-MSA), set 0 to disable
        //(*abpt).set_out_msa (0); // generate Row-Column multiple sequence alignment(RC-MSA), set 0 to disable
        (*abpt).set_out_cons (1); // generate consensus sequence, set 0 to disable
        (*abpt).w = 6;
        (*abpt).k = 9;
        (*abpt).min_w = 10; // minimizer-based seeding and partition
        (*abpt).set_progressive_poa (1);
        (*abpt).incr_fn = libc::strdup (sg.as_ptr ());
        (*abpt).set_out_gfa (if gfa { 1 } else { 0 }); // output final alignment graph in GFA format
        //(*abpt).set_out_gfa (0); // output final alignment graph in GFA format
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

        (*(*ab).abs).n_seq = n_seqs;

        align_to_reference (ab, abpt, bseqs, seq_lens);

        if (*abpt).out_msa () != 0
        {
            debug! ("outputting msa");
            abpoa_generate_rc_msa (ab, abpt);
            res.msa = Some (render_msa (ab));
        }

        if (*abpt).out_gfa () != 0
        {
            debug! ("outputting gfa");
            let mut data_ptr: *mut libc::c_char = ptr::null_mut ();
            let mut sizeloc: libc::size_t = 0;
            let out_fp = libc::open_memstream (&mut data_ptr, &mut sizeloc);
            abpoa_generate_gfa (ab, abpt, out_fp as *mut _IO_FILE);

            let gfa = file::consume_stream_to_bytes (out_fp, &mut data_ptr, &mut sizeloc)?;
            res.gfa = Some (String::from_utf8 (gfa)?);
        }

        abpoa_free(ab);
        abpoa_free_para(abpt);
    }

    Ok (res)
}

