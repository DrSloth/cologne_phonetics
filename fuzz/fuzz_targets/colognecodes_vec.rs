#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut outbuf = Vec::new();
    let _ = cologne_codes::utf8_to_cologne_codes_vec(data, &mut outbuf);
});
