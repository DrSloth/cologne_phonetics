//! Implementation of cologne codes or "Kölner Phonetik" see https://en.wikipedia.org/wiki/Cologne_phonetics
//! for more information.
//!
//! This crate is mainly used by either calling [`utf8_to_cologne_codes_vec`] or preferably creating
//! a `CologneVec` and using its [`read_from_utf8`](CologneVec::read_from_utf8) function.
//!
//! # Example
//! ```
//! # use cologne_codes::{CologneVec, CologneCode};
//! let mut buf = CologneVec::new();
//! buf.read_from_utf8("Marius Macher".as_bytes());
//! assert_eq!(buf, CologneVec::from_codes(&[
//!     CologneCode::Class6,
//!     CologneCode::Class7,
//!     CologneCode::Class8,
//!     CologneCode::Space,
//!     CologneCode::Class6,
//!     CologneCode::Class4,
//!     CologneCode::Class7,
//! ]))
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod string;
mod cologne_vec;
#[cfg(test)]
mod tests;

pub use cologne_vec::CologneVec;
pub use string::utf8_to_cologne_codes_string;

use alloc::vec::Vec;
use core::{hint, mem};

// This iterates all chars in s but ignores all non german word characters. Besides space.

/// Utf8 special bytes for german characters
const GERMAN_SPECIAL_CHAR_FIRST_BYTE: u8 = 195;
/// Utf8 second byte for german ä
const GERMAN_AE_SECOND_BYTE: u8 = 164;
/// Utf8 second byte for german ö
const GERMAN_OE_SECOND_BYTE: u8 = 182;
/// Utf8 second byte for german ü
const GERMAN_UE_SECOND_BYTE: u8 = 188;
/// Utf8 second byte for german ß
const GERMAN_SZ_SECOND_BYTE: u8 = 159;

/// Lookup code for an UNCERTAIN_X character
const UNCERTAIN_X: u8 = 13;

#[rustfmt::skip]
/// Lookups for the cologne codes as numbers uncertain characters are mapped to other numbers:
/// 9 is 'C', 10 is 'D' or 'T', 11 is 'H', 12 is 'P', 13 is 'X', Space characters are 14
const CHARACTER_TO_CODE: [u8; 27] = [
    // A  B  C  D  E  F  G  H  I  J  K  L  M
       0, 1, 9, 10,0, 3, 4, 11,0, 0, 4, 5, 6,
    // N  O  P  Q  R  S  T  U  V  W  X           Y  Z  SPACE
       6, 0, 12,4, 7, 8, 10,0, 3, 3, UNCERTAIN_X,0, 8, 14,
];
/// Slide the array one to the left
macro_rules! array_slide {
    ($arr:ident, $val:expr) => {
        $arr[0] = $arr[1];
        $arr[1] = $val;
    };
}

/// One iteration of the algorithm to be useable in both the [`CologneVec`] and the
/// [`utf8_to_cologne_codes_vec`] function
macro_rules! iter {
    ($byte: ident, $utf8:ident, $last:ident, $prev_uncertain:ident, $cologne_code_push:path, $outbuf:ident) => {
        'blk: {
            let mut b = $byte;

            if b > 0x7F {
                $utf8 = b == GERMAN_SPECIAL_CHAR_FIRST_BYTE;
                break 'blk;
            }

            if $utf8 {
                $utf8 = false;
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
                    _ => break 'blk,
                }
            }

            // Try to uppercase the letters
            b = lowercase_b(b);

            if $prev_uncertain {
                $prev_uncertain = false;
                match ($last[0], $last[1], b) {
                    // Uncertain P
                    (_, Idx::P, Idx::H) => {
                        $cologne_code_push($outbuf, CologneCode::Class3);
                    }
                    (_, Idx::P, _) => {
                        $cologne_code_push($outbuf, CologneCode::Class1);
                    }
                    // Uncertain T or D
                    (_, Idx::D | Idx::T, Idx::C | Idx::S | Idx::Z) => {
                        $cologne_code_push($outbuf, CologneCode::Class8);
                    }
                    (_, Idx::D | Idx::T, _) => {
                        $cologne_code_push($outbuf, CologneCode::Class2);
                    }
                    // Uncertain C
                    (
                        Idx::SPACE,
                        Idx::C,
                        Idx::A
                        | Idx::H
                        | Idx::K
                        | Idx::L
                        | Idx::O
                        | Idx::Q
                        | Idx::R
                        | Idx::U
                        | Idx::X,
                    ) => {
                        $cologne_code_push($outbuf, CologneCode::Class4);
                    }
                    (Idx::S | Idx::Z, Idx::C, _) => {
                        $cologne_code_push($outbuf, CologneCode::Class8);
                    }
                    (_, Idx::C, Idx::A | Idx::H | Idx::K | Idx::O | Idx::Q | Idx::U | Idx::X) => {
                        $cologne_code_push($outbuf, CologneCode::Class4);
                    }
                    (Idx::SPACE, Idx::C, _) => {
                        $cologne_code_push($outbuf, CologneCode::Class8);
                    }
                    (_, Idx::C, _) => {
                        $cologne_code_push($outbuf, CologneCode::Class8);
                    }
                    _ => {
                        unreachable!("$prev_uncertain with $last: {:?} cur: {}", $last, b)
                    }
                }
            }

            let res = *CHARACTER_TO_CODE.get(usize::from(b)).unwrap_or_else(|| {
                // SAFETY: b should never by higher than 26 so indexing into the array yields
                // always correct values.
                unsafe { hint::unreachable_unchecked() }
            });

            // eprintln!("res: {res} b: {b} $last: {$last:?}");
            match res {
                // Correct code already
                0..=8 => {
                    // SAFETY: 0..=8 are all valid Cologne codes
                    let c: CologneCode = unsafe { nibble_to_cologne(res) };
                    $cologne_code_push($outbuf, c);
                }
                UNCERTAIN_X => match $last[1] {
                    Idx::C | Idx::K | Idx::Q => {
                        $cologne_code_push($outbuf, CologneCode::Class8);
                    }
                    _ => {
                        $cologne_code_push($outbuf, CologneCode::Class4);
                        $cologne_code_push($outbuf, CologneCode::Class8);
                    }
                },
                11 => {}
                14 => {
                    $cologne_code_push($outbuf, (CologneCode::Space));
                }
                _ => {
                    $prev_uncertain = true;
                }
            }
            array_slide!($last, b);
        }
    };
}

/// Lowercase a given character, returns 26 for non ascii letter characters like punctuation etc.
fn lowercase_b(b: u8) -> u8 {
    if b < b'A' || (b > b'Z' && b < b'a') || b > b'z' {
        26
    } else {
        b.wrapping_sub(b'A') & !32
    }
}

pub(crate) use array_slide;
pub(crate) use iter;

/// Read the given utf8 bytes into the `outbuf`. Generally you should prefer using a [`CologneVec`]
pub fn utf8_to_cologne_codes_vec(bytes: &[u8], outbuf: &mut Vec<CologneCode>) {
    let mut utf8 = false;
    // All values are interpreted as a normal alphabetic character and this maps to their alphabet
    // index, most ascii punctuation and whitespace characters are 26 and count as a stop
    let mut last = [26, 26];
    // Wether the previous character was uncertain and is not yet written
    let mut prev_uncertain = false;

    for b in bytes {
        let b = *b;
        iter!(b, utf8, last, prev_uncertain, cologne_code_push, outbuf);
    }

    cologne_code_push(outbuf, CologneCode::Space);
    outbuf.pop();
}

/// Convert the `Vec<CologneCode>` to a `Vec<u8>` without any iteration or allocation.
pub fn cologne_code_vec_to_bytevec(mut outbuf: Vec<CologneCode>) -> Vec<u8> {
    let raw_ptr = outbuf.as_mut_ptr();
    let new_outbuf = unsafe {
        Vec::from_raw_parts(raw_ptr as *mut u8, outbuf.len(), outbuf.capacity())
    };
    mem::forget(outbuf);
    new_outbuf
}

/// Push a cologne code to the end of `outbuf`.
fn cologne_code_push(outbuf: &mut Vec<CologneCode>, code: CologneCode) {
    // Zero codes not after space must be overwritten
    if !outbuf.last().is_some_and(|val| *val == code) {
        match outbuf.get(outbuf.len().saturating_sub(2)..outbuf.len()) {
            Some(&[CologneCode::Space, CologneCode::Class0]) => (),
            Some(&[_, CologneCode::Class0]) => {
                *outbuf.last_mut().unwrap_or_else(|| unreachable!()) = code;
                return;
            }
            _ => (),
        }
        outbuf.push(code);
    }
}

/// Convert a given nibble to a cologne code via a transmute
///
/// # SAFETY:
/// The given byte must be a valid representation of a [`CologneCode`].
#[inline(always)]
unsafe fn nibble_to_cologne(b: u8) -> CologneCode {
    mem::transmute(b)
}

/// A representation of `CologneCode`s stored in a a nibble.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CologneCode {
    /// Cologne code represented as `0`: A, E, I, O, U, Y
    Class0 = 0b0000,
    /// Cologne code represented as `1`: B, P if not in front of H
    Class1 = 0b0001,
    /// Cologne code represented as `2`: D and T if not in front of C,S or Z
    Class2 = 0b0010,
    /// Cologne code represented as `3`: F, V, W or PH
    Class3 = 0b0011,
    /// Cologne code represented as `4`: G, K, Q, or C as first letter followed by
    /// A,H,K,L,O,Q,R,U,X or in front of A,H,K,O,Q,U,X but not after S or Z
    Class4 = 0b0100,
    /// Cologne code represented as `5`: L
    Class5 = 0b0101,
    /// Cologne code represented as `6`: M,N
    Class6 = 0b0110,
    /// Cologne code represented as `7`: R
    Class7 = 0b0111,
    /// Cologne code represented as `8`: C after S or Z, as first letter if not followed by
    /// A, H, K, L, O, Q, R, U, X or if not followed by A, H, K, O, Q, U, X
    Class8 = 0b1000,
    /// A space character or any other character which breaks the word.
    Space = 0b1110,
}

impl CologneCode {
    /// Convert this `CologneCode` to is actual byte value. Will always just be a nibble.
    #[allow(clippy::as_conversions)]
    pub const fn get(self) -> u8 {
        self as u8
    }

    /// Converts this `CologneCode` to a char
    pub const fn as_char(self) -> char {
        match self {
            Self::Class0 => '0',
            Self::Class1 => '1',
            Self::Class2 => '2',
            Self::Class3 => '3',
            Self::Class4 => '4',
            Self::Class5 => '5',
            Self::Class6 => '6',
            Self::Class7 => '7',
            Self::Class8 => '8',
            Self::Space => ' ',
        }
    }
}

impl core::fmt::Display for CologneCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

/// Namespace for alphabetic indices for letters;
struct Idx;

#[allow(dead_code)]
impl Idx {
    #![allow(clippy::missing_docs_in_private_items)]
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
    const SPACE: u8 = 26;
}

