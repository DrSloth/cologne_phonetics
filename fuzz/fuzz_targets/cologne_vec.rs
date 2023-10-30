#![no_main]

use cologne_phonetics::CologneVec;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut cv = CologneVec::new();
    let _ = cv.read_from_utf8(data);
});
