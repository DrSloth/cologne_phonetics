use alloc::string::String;

use crate::*;

/// Write characters of cologne codes 
pub fn utf8_to_cologne_codes_string(bytes: &[u8], outbuf: &mut String) {
    let mut utf8 = false;
    // All values are interpreted as a normal alphabetic character and this maps to their alphabet
    // index, most ascii punctuation and whitespace characters are 26 and count as a stop
    let mut last = [26, 26];
    // Wether the previous character was uncertain and is not yet written
    let mut prev_uncertain = false;
    let mut cologne_string = CologneString {
        inner: outbuf,
        last: [None;2],
    };
    let outbuf = &mut cologne_string;

    for b in bytes {
        let b = *b;
        iter!(b, utf8, last, prev_uncertain, cologne_code_push_char, outbuf);
    }

    match outbuf.last {
        [Some(a), Some(CologneCode::Space)] => {
            outbuf.inner.push(a.as_char());
        }
        [Some(a @ CologneCode::Space), Some(b @ CologneCode::Class0)] => {
            outbuf.inner.push(a.as_char());
            outbuf.inner.push(b.as_char());
        }
        [Some(a), Some(b)] => {
            outbuf.inner.push(a.as_char());
            if b != CologneCode::Class0 {
                outbuf.inner.push(b.as_char());
            }
        }
        [Some(CologneCode::Space), None] => (),
        [Some(a), None] => {
            outbuf.inner.push(a.as_char());
        }
        _ => ()
    }
}

/// Push a char to the end of a `CologneString` wrapper
fn cologne_code_push_char(outbuf: &mut CologneString, code: CologneCode) {
    if !outbuf.last().is_some_and(|val| val == code) {
        match &mut outbuf.last {
            &mut [Some(a), Some(CologneCode::Class0)] if a != CologneCode::Space => {
                outbuf.last[1] = Some(code);
            }
            &mut [Some(ref mut a), Some(ref mut b)] => {
                outbuf.inner.push(a.as_char());
                *a = *b;
                *b = code;
            },
            &mut [Some(_), ref mut b @ None] => {
                *b = Some(code);
            }
            &mut [ref mut a @ None, None] => {
                *a = Some(code);
            }
            &mut [None, Some(_)] => unreachable!(),
        }
    }
}

/// Small wrapper structure to push to the string efficiently
#[derive(Debug)]
struct CologneString<'a> {
    last: [Option<CologneCode>;2],
    inner: &'a mut String,
}

impl CologneString<'_> {
    /// Get the last added `CologneCode`
    fn last(&self) -> Option<CologneCode> {
        self.last[1].or(self.last[0])
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn wikipedia() {
        let mut outbuf = String::new();
        utf8_to_cologne_codes_string(b"Wikipedia", &mut outbuf);
        assert_eq!(
            outbuf,
            "3412",
        )
    }

    #[test]
    fn mueller_luedenscheidt() {
        let mut outbuf = String::new();
        utf8_to_cologne_codes_string("Müller-Lüdenscheidt".as_bytes(), &mut outbuf);
        assert_eq!(
            outbuf,
            "657 52682",
        )
    }

    #[test]
    fn breschnew() {
        let mut outbuf = String::new();
        utf8_to_cologne_codes_string("Breschnew".as_bytes(), &mut outbuf);
        assert_eq!(
            outbuf,
            "17863",
        )
    }

    #[test]
    fn veni_vidi_vici() {
        let mut outbuf = String::new();
        utf8_to_cologne_codes_string("Er kam, Er sah, Er siegte".as_bytes(), &mut outbuf);
        assert_eq!(
            outbuf,
            "07 46 07 8 07 842"
        )
    }

    #[test]
    fn special_char_spam() {
        let mut outbuf = String::new();
        utf8_to_cologne_codes_string(
            "!\"#$%&'()*+,-./0123456789:;<=>?@[\\]^_`{|}~`".as_bytes(),
            &mut outbuf,
        );
        assert_eq!(outbuf, "")
    }

    #[test]
    fn special_char_spam2() {
        let mut outbuf = String::new();
        utf8_to_cologne_codes_string(
            "a!\"#$%&'()*+,-./0123456789:;<=>?@[\\]^_`{|}~a`".as_bytes(),
            &mut outbuf,
        );
        assert_eq!(
            outbuf,
            "0 0",
        )
    }

    #[test]
    fn grundlagen() {
        let mut outbuf = String::new();
        utf8_to_cologne_codes_string("Anhand von Grundlagen".as_bytes(), &mut outbuf);
        assert_eq!(
            outbuf,
            "0662 36 4762546",
        )
    }

    #[test]
    fn hacico() {
        let mut outbuf = String::new();
        utf8_to_cologne_codes_string("Hacico".as_bytes(), &mut outbuf);
        assert_eq!(
            outbuf,
            "084",
        )
    }

    #[test]
    fn aho_aho() {
        let mut outbuf = String::new();
        utf8_to_cologne_codes_string("aho aho".as_bytes(), &mut outbuf);
        assert_eq!(
            outbuf,
            "0 0",
        )
    }
    
    #[test]
    fn aho_aho_aho() {
        let mut outbuf = String::new();
        utf8_to_cologne_codes_string("aho aho aho".as_bytes(), &mut outbuf);
        assert_eq!(
            outbuf,
            "0 0 0",
        )
    }

    #[test]
    fn aro_aro() {
        let mut outbuf = String::new();
        utf8_to_cologne_codes_string("aro aro".as_bytes(), &mut outbuf);
        assert_eq!(
            outbuf,
            "07 07",
        )
    }
    
    #[test]
    fn alphabet() {
        let mut outbuf = String::new();
        utf8_to_cologne_codes_string(
            "A B C D E F G H I J K L M N O P Q R S T U V W X Y Z".as_bytes(),
            &mut outbuf,
        );
        let resstr = "0 1 8 2 0 3 4 0 0 4 5 6 6 0 1 4 7 8 2 0 3 3 48 0 8";
        assert_eq!(outbuf, resstr);
        let mut outbuf_little = String::new();
        utf8_to_cologne_codes_string(
            "a b c d e f g h i j k l m n o p q r s t u v w x y z".as_bytes(),
            &mut outbuf_little,
        );
        assert_eq!(outbuf_little, resstr);
        assert_eq!(outbuf_little, outbuf);
    }
}

