#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use libc::sockaddr;

type __int32_t = i32;
type __int64_t = i64;
type __uint64_t = u64;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
