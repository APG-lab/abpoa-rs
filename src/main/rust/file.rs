

use crate::helper;
use libc;
use std::mem;
use std::ffi;
use std::slice;

pub fn consume_stream_to_bytes (out_fp: *mut libc::FILE, buffer: &mut *mut libc::c_char, sizeloc: &mut libc::size_t)
    -> Result<Vec<u8>, helper::PublicError>
{
    unsafe {
        libc::fclose (out_fp);
        // stream flushed, data pointer and size now up-to-date
        let b_slice: &[u8] = mem::transmute (slice::from_raw_parts (*buffer, *sizeloc + 1));
        // c_char may be i8, but ffi::CStr::from_bytes_with_nul takes u8
        let res = ffi::CStr::from_bytes_with_nul (b_slice)?.to_str ()?.as_bytes ().to_owned ();
        libc::free (*buffer as *mut libc::c_void);
        Ok (res)
    }
}
