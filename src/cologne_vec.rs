use core::{hint, ops::ControlFlow};
use alloc::vec::Vec;

use crate::*;

/// Optimized data structure to store [`CologneCode`]s.
///
/// As a single [`CologneCode`] only requires 4 bits of storage we store two in a single byte 
/// to reduce memory usage and improve cache locality.
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct CologneVec {
    /// Number of stored cologne_codes this should never overflow as 
    /// self.inner.len() < isize::MAX is guaranteed
    len: usize,
    /// The inner buffer of this `CologneVec`
    inner: Vec<u8>,
}

impl CologneVec {
    /// Create a new `CologneVec` with empty backing storage
    pub fn new() -> Self {
        Self {
            len: 0,
            inner: Vec::new(),
        }
    }

    /// Create a new `CologneVec` with a backing storage that can hald at least `cap` *bytes*.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            len: 0,
            inner: Vec::with_capacity(cap),
        }
    }

    /// Create a new `CologneVec` from the given backing storage, the storage will be cleared.
    pub fn from_inner(mut inner: Vec<u8>) -> Self {
        inner.clear();
        Self { len: 0, inner }
    }

    /// Create a new `CologneVec` from the given backing storage and a len.
    ///
    /// SAFETY:
    /// The inner vector must be initialized for atleast len CologneCodes which all have to be valid.
    pub unsafe fn from_raw(inner: Vec<u8>, len: usize) -> Self {
        Self { len, inner }
    }

    /// Create a `CologneVec` from raw [`CologneCode`]s
    pub fn from_codes(codes: &[CologneCode]) -> Self {
        let mut me = Self::new();
        for code in codes {
            me.push_raw(*code);
        }
        me.finish();
        me
    }

    /// Get the raw backign storage out this `CologneVec`
    pub fn into_inner(self) -> Vec<u8> {
        self.inner
    }
    
    /// Get the backing storage and the current len from this `CologneVec`
    pub fn into_raw(self) -> (Vec<u8>, usize) {
        (self.inner, self.len)
    }

    /// Get the number of stored [`CologneCode`]s
    pub fn len(&self) -> usize {
        self.len
    }

    /// Get the raw backing storage as bytes
    pub fn get_raw(&self) -> &[u8] {
        &self.inner
    }

    /// Primary entry point. Convert the given raw text bytes into [`CologneCode`]s.
    ///
    /// This function does not allocate any new storage but might reallocate the internal buffer.
    pub fn read_from_utf8(&mut self, bytes: &[u8]) {
        // Naive iteration
        let mut utf8 = false;
        // All values are interpreted as a normal alphabetic character and this maps to their alphabet
        // index, most ascii punctuation and whitespace characters are 26 and count as a stop
        let mut last = [26, 26];
        // Wether the previous character was uncertain and is not yet written
        let mut prev_uncertain = false;

        for b in bytes {
            let b = *b;
            crate::iter!(b, utf8, last, prev_uncertain, CologneVec::push, self);
        }

        self.finish()
    }

    /// Push a new [`CologneCode`] to the end of this `CologneVec` according to the rules of how
    /// cologne codes have to be created. This automatically dedups codes next to each other.
    #[inline(always)]
    pub fn push(&mut self, code: CologneCode) {
        if !self.last_is(code) {
            let last = self.last_byte();
            if self.len >= 2 && (last & 0x0f) == 0 && (last >> 4) != CologneCode::Space.get() {
                self.replace_last(code);
            } else {
                self.push_raw(code);
            }
        }
    }

    /// Push to the end of the [`CologneVec`] without any other checks.
    #[inline(always)]
    fn push_raw(&mut self, code: CologneCode) {
        if Self::byte_bound(self.len) {
            self.inner.push(code.get() << 4)
        } else {
            if let Some(last) = self.inner.last_mut() {
                *last |= code.get();
            }
        }
        self.len = self.len.wrapping_add(1);
    }

    /// Check if the stored [`CologneCode`]s are currently bound to a byte border.
    #[inline(always)]
    const fn byte_bound(len: usize) -> bool {
        len & 0x01 == 0
    }

    /// Check if the last stored [`CologneCode`] is equal to the given [`CologneCode`] `code`
    #[inline(always)]
    fn last_is(&self, code: CologneCode) -> bool {
        let code_hi = code.get() << 4;
        if let Some(last) = self.inner.last() {
            if Self::byte_bound(self.len) {
                last << 4 == code_hi
            } else {
                *last == code_hi
            }
        } else {
            false
        }
    }

    /// Get the last stored cologne code
    pub fn last(&self) -> Option<CologneCode> {
        if Self::byte_bound(self.len) {
            if self.len == 0 {
                None
            } else {
                let last = unsafe { *self.inner.get_unchecked(self.inner.len().wrapping_sub(1)) };
                let code = unsafe { nibble_to_cologne(last & 0x0f) };
                Some(code)
            }
        } else {
            let last = unsafe { *self.inner.get_unchecked(self.inner.len().wrapping_sub(1)) };
            let code = unsafe { nibble_to_cologne(last >> 4) };
            Some(code)
        }
    }

    /// Replace the last stored [`CologneCode`] with the given `code`.
    fn replace_last(&mut self, code: CologneCode) {
        if let Some(last) = self.inner.last_mut() {
            if Self::byte_bound(self.len) {
                *last &= 0xf0;
                *last |= code.get();
            } else {
                *last &= 0x0f;
                *last |= code.get() << 4;
            }
        }
    }

    /// Get the last byte consisting of the last two stored [`CologneCode`]s. Uninitialized codes
    /// are propagated as `0`.
    fn last_byte(&self) -> u8 {
        if Self::byte_bound(self.len) {
            self.inner.last().copied().unwrap_or(0)
        } else {
            let last = unsafe { self.inner.get_unchecked(self.inner.len() - 1) } >> 4;
            if self.len == 1 {
                last
            } else {
                (unsafe { *self.inner.get_unchecked(self.inner.len() - 2) } << 4) | last
            }
        }
    }

    /// Finish this `CologneVec` by applying the rules on the last element.
    pub fn finish(&mut self) {
        let last_byte = self.last_byte();
        if let Some(l) = self.inner.last_mut() {
            if last_byte == CologneCode::Space.get() << 4 | CologneCode::Class0.get() {
                return;
            }
            
            if Self::byte_bound(self.len) {
                let nib = *l & 0x0f;
                if nib == CologneCode::Class0.get() || nib == CologneCode::Space.get() {
                    self.len = self.len.wrapping_sub(1);
                    *l &= 0xf0;
                }
            } else if !Self::byte_bound(self.len) {
                let nib = *l >> 4;
                if nib == CologneCode::Class0.get() || nib == CologneCode::Space.get() {
                    self.len = self.len.wrapping_sub(1);
                    self.inner.pop();
                }
            }
        }
    }

    /// Iterate all [`CologneCode`]s with internal iteration.
    pub fn internal_iter(&self, mut f: impl FnMut(CologneCode) -> ControlFlow<()>) {
        for b in self
            .inner
            .get(0..self.inner.len().wrapping_sub(1))
            .into_iter()
            .flatten()
        {
            let hi = unsafe { nibble_to_cologne(*b >> 4) };
            let lo = unsafe { nibble_to_cologne(*b & 0x0f) };
            if f(hi).is_break() {
                return;
            }
            if f(lo).is_break() {
                return;
            }
        }

        if let Some(last) = self.inner.last() {
            if Self::byte_bound(self.len) {
                let hi = unsafe { nibble_to_cologne(*last >> 4) };
                let lo = unsafe { nibble_to_cologne(*last & 0x0f) };
                if f(hi).is_break() {
                    return;
                }
                f(lo);
            } else {
                let hi = unsafe { nibble_to_cologne(*last >> 4) };
                f(hi);
            }
        }
    }

    /// Clear this [`CologneVec`]
    pub fn clear(&mut self) {
        self.inner.clear();
        self.len = 0;
    }
}

impl core::fmt::Debug for CologneVec {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[")?;
        self.internal_iter(|code| match write!(f, "{}", code) {
            Ok(()) => ControlFlow::Continue(()),
            Err(_) => ControlFlow::Break(()),
        });
        write!(f, "]")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn wikipedia() {
        let mut outbuf = CologneVec::new();
        outbuf.read_from_utf8(b"Wikipedia");
        assert_eq!(
            outbuf,
            CologneVec::from_codes(&[
                CologneCode::Class3,
                CologneCode::Class4,
                CologneCode::Class1,
                CologneCode::Class2,
            ])
        )
    }

    #[test]
    fn mueller_luedenscheidt() {
        let mut outbuf = CologneVec::new();
        outbuf.read_from_utf8("Müller-Lüdenscheidt".as_bytes());
        assert_eq!(
            outbuf,
            CologneVec::from_codes(&[
                CologneCode::Class6,
                CologneCode::Class5,
                CologneCode::Class7,
                CologneCode::Space,
                CologneCode::Class5,
                CologneCode::Class2,
                CologneCode::Class6,
                CologneCode::Class8,
                CologneCode::Class2,
            ])
        )
    }

    #[test]
    fn breschnew() {
        let mut outbuf = CologneVec::new();
        outbuf.read_from_utf8("Breschnew".as_bytes());
        assert_eq!(
            outbuf,
            CologneVec::from_codes(&[
                CologneCode::Class1,
                CologneCode::Class7,
                CologneCode::Class8,
                CologneCode::Class6,
                CologneCode::Class3,
            ])
        )
    }

    #[test]
    fn veni_vidi_vici() {
        let mut outbuf = CologneVec::new();
        outbuf.read_from_utf8("Er kam, Er sah, Er siegte".as_bytes());
        assert_eq!(
            outbuf,
            CologneVec::from_codes(&[
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
            ])
        )
    }

    #[test]
    fn special_char_spam() {
        let mut outbuf = CologneVec::new();
        outbuf.read_from_utf8("!\"#$%&'()*+,-./0123456789:;<=>?@[\\]^_`{|}~`".as_bytes());
        assert_eq!(outbuf, CologneVec::from_codes(&[]))
    }

    #[test]
    fn special_char_spam2() {
        let mut outbuf = CologneVec::new();
        outbuf.read_from_utf8("a!\"#$%&'()*+,-./0123456789:;<=>?@[\\]^_`{|}~a`".as_bytes());
        assert_eq!(
            outbuf,
            CologneVec::from_codes(
                &[CologneCode::Class0, CologneCode::Space, CologneCode::Class0,]
            )
        )
    }

    #[test]
    fn grundlagen() {
        let mut outbuf = CologneVec::new();
        outbuf.read_from_utf8("Anhand von Grundlagen".as_bytes());
        assert_eq!(
            outbuf,
            CologneVec::from_codes(&[
                CologneCode::Class0,
                CologneCode::Class6,
                CologneCode::Class6,
                CologneCode::Class2,
                CologneCode::Space,
                CologneCode::Class3,
                CologneCode::Class6,
                CologneCode::Space,
                CologneCode::Class4,
                CologneCode::Class7,
                CologneCode::Class6,
                CologneCode::Class2,
                CologneCode::Class5,
                CologneCode::Class4,
                CologneCode::Class6,
            ])
        )
    }

    #[test]
    fn hacico() {
        let mut outbuf = CologneVec::new();
        outbuf.read_from_utf8("Hacico".as_bytes());
        assert_eq!(
            outbuf,
            CologneVec::from_codes(&[
                CologneCode::Class0,
                CologneCode::Class8,
                CologneCode::Class4,
            ])
        )
    }

    #[test]
    fn aho_aho() {
        let mut outbuf = CologneVec::new();
        outbuf.read_from_utf8("aho aho".as_bytes());
        assert_eq!(
            outbuf,
            CologneVec::from_codes(&[
                CologneCode::Class0,
                CologneCode::Space,
                CologneCode::Class0,
            ])
        )
    }

    #[test]
    fn aho_aho_aho() {
        let mut outbuf = CologneVec::new();
        outbuf.read_from_utf8("aho aho aho".as_bytes());
        assert_eq!(
            outbuf,
            CologneVec::from_codes(&[
                CologneCode::Class0,
                CologneCode::Space,
                CologneCode::Class0,
                CologneCode::Space,
                CologneCode::Class0,
            ])
        )
    }

    #[test]
    fn aro_aro() {
        let mut outbuf = CologneVec::new();
        outbuf.read_from_utf8("aro aro".as_bytes());
        assert_eq!(
            outbuf,
            CologneVec::from_codes(&[
                CologneCode::Class0,
                CologneCode::Class7,
                CologneCode::Space,
                CologneCode::Class0,
                CologneCode::Class7,
            ])
        )
    }

    #[test]
    fn alphabet() {
        let mut outbuf = CologneVec::new();
        outbuf.read_from_utf8("A B C D E F G H I J K L M N O P Q R S T U V W X Y Z".as_bytes());
        let resvec = CologneVec::from_codes(&[
            CologneCode::Class0, // A
            CologneCode::Space,
            CologneCode::Class1, // B
            CologneCode::Space,
            CologneCode::Class8, // C
            CologneCode::Space,
            CologneCode::Class2, // D
            CologneCode::Space,
            CologneCode::Class0, // E
            CologneCode::Space,
            CologneCode::Class3, // F
            CologneCode::Space,
            CologneCode::Class4, // G
            CologneCode::Space,
            // H is ignored
            CologneCode::Class0, // I
            CologneCode::Space,
            CologneCode::Class0, // J
            CologneCode::Space,
            CologneCode::Class4, // K
            CologneCode::Space,
            CologneCode::Class5, // L
            CologneCode::Space,
            CologneCode::Class6, // M
            CologneCode::Space,
            CologneCode::Class6, // N
            CologneCode::Space,
            CologneCode::Class0, // O
            CologneCode::Space,
            CologneCode::Class1, // P
            CologneCode::Space,
            CologneCode::Class4, // Q
            CologneCode::Space,
            CologneCode::Class7, // R
            CologneCode::Space,
            CologneCode::Class8, // S
            CologneCode::Space,
            CologneCode::Class2, // T
            CologneCode::Space,
            CologneCode::Class0, // U
            CologneCode::Space,
            CologneCode::Class3, // V
            CologneCode::Space,
            CologneCode::Class3, // W
            CologneCode::Space,
            CologneCode::Class4, // X
            CologneCode::Class8, // X
            CologneCode::Space,
            CologneCode::Class0, // Y
            CologneCode::Space,
            CologneCode::Class8, // Z
        ]);
        assert_eq!(outbuf, resvec);
        let mut outbuf_little = CologneVec::new();
        outbuf_little
            .read_from_utf8("a b c d e f g h i j k l m n o p q r s t u v w x y z".as_bytes());
        assert_eq!(outbuf_little, resvec);
        assert_eq!(outbuf_little, outbuf);
    }
}
