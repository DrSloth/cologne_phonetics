#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut outbuf = String::new();
    let _ = cologne_phonetics::utf8_to_cologne_phonetics_string(data, &mut outbuf);
});
