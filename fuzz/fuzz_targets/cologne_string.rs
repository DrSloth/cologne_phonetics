#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut outbuf = String::new();
    let _ = cologne_codes::utf8_to_cologne_codes_string(data, &mut outbuf);
});
