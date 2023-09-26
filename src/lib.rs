/// Implementation of cologne codes or "Kölner Phonetik" see https://en.wikipedia.org/wiki/Cologne_phonetics
/// for more information
use std::{fmt::Display, mem};

// This iterates all chars in s but ignores all non german word characters. Besides space.

const GERMAN_SPECIAL_CHAR_FIRST_BYTE: u8 = 195;
const GERMAN_AE_SECOND_BYTE: u8 = 164;
const GERMAN_OE_SECOND_BYTE: u8 = 182;
const GERMAN_UE_SECOND_BYTE: u8 = 188;
const GERMAN_SZ_SECOND_BYTE: u8 = 159;

const UNCERTAIN_X: u8 = 13;

#[rustfmt::skip]
/// Lookups for the cologne codes as numbers uncertain characters are mapped to other numbers:
/// 9 is 'C', 10 is 'D' or 'T', 11 is 'H', 12 is 'P', 13 is 'X'
const CHARACTER_TO_CODE: [u8; 26] = [
    // A  B  C  D  E  F  G  H  I  J  K  L  M
       0, 1, 9, 10,0, 3, 4, 11,0, 0, 4, 5, 6,
    // N  O  P  Q  R  S  T  U  V  W  X           Y  Z
       6, 0, 12,0, 7, 8, 10,0, 3, 3, UNCERTAIN_X,0, 8
];

macro_rules! array_slide {
    ($arr:ident, $val:expr) => {
        $arr[0] = $arr[1];
        $arr[1] = $val;
    };
}

pub fn utf8_to_cologne_codes(bytes: &[u8], outbuf: &mut Vec<CologneCode>) {
    // Naive iteration
    let mut utf8 = false;
    // All values are interpreted as a normal alphabetic character and this maps to their alphabet
    // index, most ascii punctuation and whitespace characters are 26 and count as a stop
    let mut last = [26, 26];
    // Wether the previous character was uncertain and is not yet written
    let mut prev_uncertain = false;

    for b in bytes {
        let _c = *b;
        let mut b = *b;
        if b > 0x7F {
            utf8 = b == GERMAN_SPECIAL_CHAR_FIRST_BYTE;
            continue;
        }

        if utf8 {
            utf8 = false;
            match b {
                GERMAN_AE_SECOND_BYTE => {
                    b = b'A';
                }
                GERMAN_OE_SECOND_BYTE => {
                    b = b'O';
                }
                GERMAN_UE_SECOND_BYTE => {
                    b = b'U';
                }
                GERMAN_SZ_SECOND_BYTE => {
                    b = b'Z';
                }
                _ => continue,
            }
        }

        // Try to uppercase the letters
        if b < b'A' {
            // Stop character
            array_slide!(last, 26);
            cologne_code_push(CologneCode::Space, outbuf);
            utf8 = false;
            continue;
        }
        if b > b'Z' {
            b = b.wrapping_sub(b'a' - b'A');
            if b > b'Z' || b < b'A' {
                utf8 = false;
                // Stop character
                array_slide!(last, 26);
                cologne_code_push(CologneCode::Space, outbuf);
                continue;
            }
        }
        b = b.wrapping_sub(b'A');

        if prev_uncertain {
            prev_uncertain = false;
            // TODO This match can probably be optimized
            match (last[0], last[1], b) {
                // Uncertain P
                (_, Idx::P, Idx::H) => {
                    cologne_code_push(CologneCode::Class3, outbuf);
                }
                (_, Idx::P, _) => {
                    cologne_code_push(CologneCode::Class1, outbuf);
                }
                // Uncertain T or D
                (_, Idx::D | Idx::T, Idx::C | Idx::S | Idx::Z) => {
                    cologne_code_push(CologneCode::Class8, outbuf);
                }
                (_, Idx::D | Idx::T, _) => {
                    cologne_code_push(CologneCode::Class2, outbuf);
                }
                // Uncertain C
                (
                    Idx::STOP,
                    Idx::C,
                    Idx::A | Idx::H | Idx::K | Idx::L | Idx::O | Idx::Q | Idx::R | Idx::U | Idx::X,
                ) => {
                    cologne_code_push(CologneCode::Class4, outbuf);
                }
                (Idx::S | Idx::Z, Idx::C, _) => {
                    cologne_code_push(CologneCode::Class8, outbuf);
                }
                (_, Idx::C, Idx::A | Idx::H | Idx::K | Idx::O | Idx::Q | Idx::U | Idx::X) => {
                    cologne_code_push(CologneCode::Class8, outbuf);
                }
                (Idx::STOP, Idx::C, _) => {
                    cologne_code_push(CologneCode::Class8, outbuf);
                }
                (_, Idx::C, _) => {
                    cologne_code_push(CologneCode::Class8, outbuf);
                }
                _ => {
                    unreachable!("prev_uncertain with last: {:?} cur: {}", last, b)
                }
            }
        }

        let res = *CHARACTER_TO_CODE
            .get(usize::from(b))
            .unwrap_or_else(|| unreachable!());
        match res {
            // Correct code already
            0 if last[1] == 26 => {
                let c: CologneCode = unsafe { mem::transmute(res) };
                cologne_code_push(c, outbuf);
            }
            0 => {
                // Zeroes not after space are ignored
                ()
            }
            1..=8 => {
                let c: CologneCode = unsafe { mem::transmute(res) };
                cologne_code_push(c, outbuf);
            }
            UNCERTAIN_X => match last[1] {
                Idx::C | Idx::K | Idx::Q => {
                    cologne_code_push(CologneCode::Class8, outbuf);
                }
                _ => {
                    cologne_code_push(CologneCode::Class4, outbuf);
                    cologne_code_push(CologneCode::Class8, outbuf);
                }
            },
            11 => {}
            _ => {
                prev_uncertain = true;
            }
        }
        array_slide!(last, b);
    }
}

fn cologne_code_push(code: CologneCode, outbuf: &mut Vec<CologneCode>) {
    if !outbuf.last().map(|val| *val == code).unwrap_or(false) {
        outbuf.push(code)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum CologneCode {
    Class0 = 0b0000,
    Class1 = 0b0001,
    Class2 = 0b0010,
    Class3 = 0b0011,
    Class4 = 0b0100,
    Class5 = 0b0101,
    Class6 = 0b0110,
    Class7 = 0b0111,
    Class8 = 0b1000,
    Space = 0b1110,
    Stop = 0b1111,
}

impl CologneCode {
    pub const fn get(self) -> u8 {
        self as u8
    }
}

impl Display for CologneCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Class0 => write!(f, "0"),
            Self::Class1 => write!(f, "1"),
            Self::Class2 => write!(f, "2"),
            Self::Class3 => write!(f, "3"),
            Self::Class4 => write!(f, "4"),
            Self::Class5 => write!(f, "5"),
            Self::Class6 => write!(f, "6"),
            Self::Class7 => write!(f, "7"),
            Self::Class8 => write!(f, "8"),
            Self::Space => write!(f, " "),
            Self::Stop => Ok(()),
        }
    }
}

/// SAFETY: Only unsafe if the unsafe_opt feature is on
fn slice_to_arr<T, const N: usize>(s: &[T]) -> &[T; N] {
    <&[T] as TryInto<&[T; N]>>::try_into(s).unwrap_or_else(|_| {
        // SAFETY: only unsafe if unsafe_opt is on
        // unsafe { unreachable() }
        unreachable!()
    })
}

struct Idx;

#[allow(dead_code)]
impl Idx {
    const A: u8 = 0;
    const B: u8 = 1;
    const C: u8 = 2;
    const D: u8 = 3;
    const E: u8 = 4;
    const F: u8 = 5;
    const G: u8 = 6;
    const H: u8 = 7;
    const I: u8 = 8;
    const J: u8 = 9;
    const K: u8 = 10;
    const L: u8 = 11;
    const M: u8 = 12;
    const N: u8 = 13;
    const O: u8 = 14;
    const P: u8 = 15;
    const Q: u8 = 16;
    const R: u8 = 17;
    const S: u8 = 18;
    const T: u8 = 19;
    const U: u8 = 20;
    const V: u8 = 21;
    const W: u8 = 22;
    const X: u8 = 23;
    const Y: u8 = 24;
    const Z: u8 = 25;
    const STOP: u8 = 26;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn wikipedia() {
        let mut outbuf = Vec::new();
        utf8_to_cologne_codes(b"Wikipedia", &mut outbuf);
        assert_eq!(
            outbuf,
            vec![
                CologneCode::Class3,
                CologneCode::Class4,
                CologneCode::Class1,
                CologneCode::Class2,
            ]
        )
    }

    #[test]
    fn mueller_luedenscheid() {
        let mut outbuf = Vec::new();
        utf8_to_cologne_codes("Müller-Lüdenscheidt".as_bytes(), &mut outbuf);
        assert_eq!(
            outbuf,
            vec![
                CologneCode::Class6,
                CologneCode::Class5,
                CologneCode::Class7,
                CologneCode::Space,
                CologneCode::Class5,
                CologneCode::Class2,
                CologneCode::Class6,
                CologneCode::Class8,
                CologneCode::Class2,
            ]
        )
    }

    #[test]
    fn breschnew() {
        let mut outbuf = Vec::new();
        utf8_to_cologne_codes("Breschnew".as_bytes(), &mut outbuf);
        assert_eq!(
            outbuf,
            vec![
                CologneCode::Class1,
                CologneCode::Class7,
                CologneCode::Class8,
                CologneCode::Class6,
                CologneCode::Class3,
            ]
        )
    }

    #[test]
    fn veni_vidi_vici() {
        let mut outbuf = Vec::new();
        utf8_to_cologne_codes("Er kam, Er sah, Er siegte".as_bytes(), &mut outbuf);
        assert_eq!(
            outbuf,
            vec![
                // Er
                CologneCode::Class0,
                CologneCode::Class7,
                CologneCode::Space,
                // Kam
                CologneCode::Class4,
                CologneCode::Class6,
                CologneCode::Space,
                // Er
                CologneCode::Class0,
                CologneCode::Class7,
                CologneCode::Space,
                // sah
                CologneCode::Class8,
                CologneCode::Space,
                // Er
                CologneCode::Class0,
                CologneCode::Class7,
                CologneCode::Space,
                // siegte
                CologneCode::Class8,
                CologneCode::Class4,
                CologneCode::Class2,
            ]
        )
    }

    #[test]
    fn special_char_spam() {
        let mut outbuf = Vec::new();
        utf8_to_cologne_codes(
            "!\"#$%&'()*+,-./0123456789:;<=>?@[\\]^_`{|}~`".as_bytes(),
             &mut outbuf
        );
        assert_eq!(
            outbuf,
            vec![
                CologneCode::Space
            ]
        )
    }
}
