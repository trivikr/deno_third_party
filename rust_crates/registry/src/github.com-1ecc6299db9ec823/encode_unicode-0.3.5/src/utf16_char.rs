/* Copyright 2016 The encode_unicode Developers
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use utf16_iterators::Utf16Iterator;
use traits::{CharExt, U16UtfExt};
use utf8_char::Utf8Char;
use errors::{InvalidUtf16Slice, InvalidUtf16Tuple, NonBMPError, EmptyStrError, FromStrError};
extern crate core;
use self::core::{hash,fmt};
use self::core::cmp::Ordering;
use self::core::borrow::Borrow;
use self::core::ops::Deref;
use self::core::str::FromStr;
#[cfg(feature="std")]
use self::core::iter::FromIterator;
#[cfg(feature="std")]
#[allow(deprecated)]
use std::ascii::AsciiExt;
#[cfg(feature="ascii")]
use self::core::char;
#[cfg(feature="ascii")]
extern crate ascii;
#[cfg(feature="ascii")]
use self::ascii::{AsciiChar,ToAsciiChar,ToAsciiCharError};


// I don't think there is any good default value for char, but char does.
#[derive(Default)]
// char doesn't do anything more advanced than u32 for Eq/Ord, so we shouldn't either.
// When it's a single unit, the second is zero, so Eq works.
// #[derive(Ord)] however, breaks on surrogate pairs.
#[derive(PartialEq,Eq)]
#[derive(Clone,Copy)]


/// An unicode codepoint stored as UTF-16.
///
/// It can be borrowed as an `u16` slice, and has the same size as `char`.
pub struct Utf16Char {
    units: [u16; 2],
}


  /////////////////////
 //conversion traits//
/////////////////////
impl FromStr for Utf16Char {
    type Err = FromStrError;
    /// Create an `Utf16Char` from a string slice.
    /// The string must contain exactly one codepoint.
    ///
    /// # Examples
    ///
    /// ```
    /// use encode_unicode::error::FromStrError::*;
    /// use encode_unicode::Utf16Char;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(Utf16Char::from_str("a"), Ok(Utf16Char::from('a')));
    /// assert_eq!(Utf16Char::from_str("🂠"), Ok(Utf16Char::from('🂠')));
    /// assert_eq!(Utf16Char::from_str(""), Err(Empty));
    /// assert_eq!(Utf16Char::from_str("ab"), Err(MultipleCodepoints));
    /// assert_eq!(Utf16Char::from_str("é"), Err(MultipleCodepoints));// 'e'+u301 combining mark
    /// ```
    fn from_str(s: &str) -> Result<Self, FromStrError> {
        match Utf16Char::from_str_start(s) {
            Ok((u16c,bytes)) if bytes == s.len() => Ok(u16c),
            Ok((_,_)) => Err(FromStrError::MultipleCodepoints),
            Err(EmptyStrError) => Err(FromStrError::Empty),
        }
    }
}
impl From<char> for Utf16Char {
    fn from(c: char) -> Self {
        let (first, second) = c.to_utf16_tuple();
        Utf16Char{ units: [first, second.unwrap_or(0)] }
    }
}
impl From<Utf8Char> for Utf16Char {
    fn from(utf8: Utf8Char) -> Utf16Char {
        let (b, utf8_len) = utf8.to_array();
        match utf8_len {
            1 => Utf16Char{ units: [b[0] as u16, 0] },
            4 => {// need surrogate
                let mut first = 0xd800 - (0x01_00_00u32 >> 10) as u16;
                first += (b[0] as u16 & 0x07) << 8;
                first += (b[1] as u16 & 0x3f) << 2;
                first += (b[2] as u16 & 0x30) >> 4;
                let mut second = 0xdc00;
                second |= (b[2] as u16 & 0x0f) << 6;
                second |=  b[3] as u16 & 0x3f;
                Utf16Char{ units: [first, second] }
            },
            _ => { // 2 or 3
                let mut unit = ((b[0] as u16 & 0x1f) << 6) | (b[1] as u16 & 0x3f);
                if utf8_len == 3 {
                    unit = (unit << 6) | (b[2] as u16 & 0x3f);
                }
                Utf16Char{ units: [unit, 0] }
            },
        }
    }
}
impl From<Utf16Char> for char {
    fn from(uc: Utf16Char) -> char {
        unsafe{ char::from_utf16_tuple_unchecked(uc.to_tuple()) }
    }
}
impl IntoIterator for Utf16Char {
    type Item=u16;
    type IntoIter=Utf16Iterator;
    /// Iterate over the units.
    fn into_iter(self) -> Utf16Iterator {
        Utf16Iterator::from(self)
    }
}

#[cfg(feature="std")]
impl Extend<Utf16Char> for Vec<u16> {
    fn extend<I:IntoIterator<Item=Utf16Char>>(&mut self,  iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        for u16c in iter {
            self.push(u16c.units[0]);
            if u16c.units[1] != 0 {
                self.push(u16c.units[1]);
            }
        }
    }
}
#[cfg(feature="std")]
impl<'a> Extend<&'a Utf16Char> for Vec<u16> {
    fn extend<I:IntoIterator<Item=&'a Utf16Char>>(&mut self,  iter: I) {
        self.extend(iter.into_iter().cloned())
    }
}
#[cfg(feature="std")]
impl FromIterator<Utf16Char> for Vec<u16> {
    fn from_iter<I:IntoIterator<Item=Utf16Char>>(iter: I) -> Self {
        let mut vec = Vec::new();
        vec.extend(iter);
        return vec;
    }
}
#[cfg(feature="std")]
impl<'a> FromIterator<&'a Utf16Char> for Vec<u16> {
    fn from_iter<I:IntoIterator<Item=&'a Utf16Char>>(iter: I) -> Self {
        Self::from_iter(iter.into_iter().cloned())
    }
}

#[cfg(feature="std")]
impl Extend<Utf16Char> for String {
    fn extend<I:IntoIterator<Item=Utf16Char>>(&mut self,  iter: I) {
        self.extend(iter.into_iter().map(|u16c| Utf8Char::from(u16c) ));
    }
}
#[cfg(feature="std")]
impl<'a> Extend<&'a Utf16Char> for String {
    fn extend<I:IntoIterator<Item=&'a Utf16Char>>(&mut self,  iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}
#[cfg(feature="std")]
impl FromIterator<Utf16Char> for String {
    fn from_iter<I:IntoIterator<Item=Utf16Char>>(iter: I) -> Self {
        let mut s = String::new();
        s.extend(iter);
        return s;
    }
}
#[cfg(feature="std")]
impl<'a> FromIterator<&'a Utf16Char> for String {
    fn from_iter<I:IntoIterator<Item=&'a Utf16Char>>(iter: I) -> Self {
        Self::from_iter(iter.into_iter().cloned())
    }
}


  /////////////////
 //getter traits//
/////////////////
impl AsRef<[u16]> for Utf16Char {
    #[inline]
    fn as_ref(&self) -> &[u16] {
        &self.units[..self.len()]
    }
}
impl Borrow<[u16]> for Utf16Char {
    #[inline]
    fn borrow(&self) -> &[u16] {
        self.as_ref()
    }
}
impl Deref for Utf16Char {
    type Target = [u16];
    #[inline]
    fn deref(&self) -> &[u16] {
        self.as_ref()
    }
}


  ////////////////
 //ascii traits//
////////////////
#[cfg(feature="std")]
#[allow(deprecated)]
impl AsciiExt for Utf16Char {
    type Owned = Self;
    fn is_ascii(&self) -> bool {
        self.units[0] < 128
    }
    fn eq_ignore_ascii_case(&self,  other: &Self) -> bool {
        self.to_ascii_lowercase() == other.to_ascii_lowercase()
    }
    fn to_ascii_uppercase(&self) -> Self {
        let n = self.units[0].wrapping_sub(b'a' as u16);
        if n < 26 {Utf16Char{ units: [n+b'A' as u16, 0] }}
        else      {*self}
    }
    fn to_ascii_lowercase(&self) -> Self {
        let n = self.units[0].wrapping_sub(b'A' as u16);
        if n < 26 {Utf16Char{ units: [n+b'a' as u16, 0] }}
        else      {*self}
    }
    fn make_ascii_uppercase(&mut self) {
        *self = self.to_ascii_uppercase()
    }
    fn make_ascii_lowercase(&mut self) {
        *self = self.to_ascii_lowercase();
    }
}

#[cfg(feature="ascii")]
/// Requires the feature "ascii".
impl From<AsciiChar> for Utf16Char {
    #[inline]
    fn from(ac: AsciiChar) -> Self {
        Utf16Char{ units: [ac.as_byte() as u16, 0] }
    }
}
#[cfg(feature="ascii")]
/// Requires the feature "ascii".
impl ToAsciiChar for Utf16Char {
    #[inline]
    fn to_ascii_char(self) -> Result<AsciiChar, ToAsciiCharError> {
        unsafe{ AsciiChar::from(char::from_u32_unchecked(self.units[0] as u32)) }
    }
    #[inline]
    unsafe fn to_ascii_char_unchecked(self) -> AsciiChar {
        AsciiChar::from_unchecked(self.units[0] as u8)
    }
}


  /////////////////////////////////////////////////////////
 //Genaral traits that cannot be derived to emulate char//
/////////////////////////////////////////////////////////
impl hash::Hash for Utf16Char {
    fn hash<H : hash::Hasher>(&self,  state: &mut H) {
        self.to_char().hash(state);
    }
}
impl fmt::Debug for Utf16Char {
    fn fmt(&self,  fmtr: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.to_char(), fmtr)
    }
}
impl fmt::Display for Utf16Char {
    fn fmt(&self,  fmtr: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&Utf8Char::from(*self), fmtr)
    }
}
// Cannot derive these impls because two-unit characters must always compare
// greater than one-unit ones.
impl PartialOrd for Utf16Char {
    #[inline]
    fn partial_cmp(&self,  rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}
impl Ord for Utf16Char {
    #[inline]
    fn cmp(&self,  rhs: &Self) -> Ordering {
        // Shift the first unit by 0xd if surrogate, and 0 otherwise.
        // This ensures surrogates are always greater than 0xffff, and
        // that the second unit only affect the result when the first are equal.
        // Multiplying by a constant factor isn't enough because that factor
        // would have to be greater than 1023 and smaller than 5.5.
        // This transformation is less complicated than combine_surrogates().
        let lhs = (self.units[0] as u32, self.units[1] as u32);
        let rhs = (rhs.units[0] as u32, rhs.units[1] as u32);
        let lhs = (lhs.0 << (lhs.1 >> 12)) + lhs.1;
        let rhs = (rhs.0 << (rhs.1 >> 12)) + rhs.1;
        lhs.cmp(&rhs)
    }
}


  ////////////////////////////////
 //Comparisons with other types//
////////////////////////////////
impl PartialEq<char> for Utf16Char {
    fn eq(&self,  u32c: &char) -> bool {
        *self == Utf16Char::from(*u32c)
    }
}
impl PartialEq<Utf16Char> for char {
    fn eq(&self,  u16c: &Utf16Char) -> bool {
        Utf16Char::from(*self) == *u16c
    }
}
impl PartialOrd<char> for Utf16Char {
    fn partial_cmp(&self,  u32c: &char) -> Option<Ordering> {
        self.partial_cmp(&Utf16Char::from(*u32c))
    }
}
impl PartialOrd<Utf16Char> for char {
    fn partial_cmp(&self,  u16c: &Utf16Char) -> Option<Ordering> {
        Utf16Char::from(*self).partial_cmp(u16c)
    }
}

impl PartialEq<Utf8Char> for Utf16Char {
    fn eq(&self,  u8c: &Utf8Char) -> bool {
        *self == Utf16Char::from(*u8c)
    }
}
impl PartialOrd<Utf8Char> for Utf16Char {
    fn partial_cmp(&self,  u8c: &Utf8Char) -> Option<Ordering> {
        self.partial_cmp(&Utf16Char::from(*u8c))
    }
}
// The other direction reverse impls in utf8_char.rs

/// Only considers the unit equal if the codepoint of the `Utf16Char` is not
/// made up of a surrogate pair.
///
/// There is no impl in the opposite direction, as this should only be used to
/// compare `Utf16Char`s against constants.
///
/// # Examples
///
/// ```
/// # use encode_unicode::Utf16Char;
/// assert!(Utf16Char::from('6') == b'6' as u16);
/// assert!(Utf16Char::from('\u{FFFF}') == 0xffff_u16);
/// assert!(Utf16Char::from_tuple((0xd876, Some(0xdef9))).unwrap() != 0xd876_u16);
/// ```
impl PartialEq<u16> for Utf16Char {
    fn eq(&self,  unit: &u16) -> bool {
        self.units[0] == *unit  &&  self.units[1] == 0
    }
}
/// Only considers the byte equal if the codepoint of the `Utf16Char` is <= U+FF.
///
/// # Examples
///
/// ```
/// # use encode_unicode::Utf16Char;
/// assert!(Utf16Char::from('6') == b'6');
/// assert!(Utf16Char::from('\u{00FF}') == b'\xff');
/// assert!(Utf16Char::from('\u{0100}') != b'\0');
/// ```
impl PartialEq<u8> for Utf16Char {
    fn eq(&self,  byte: &u8) -> bool {
        self.units[0] == *byte as u16
    }
}
#[cfg(feature = "ascii")]
/// `Utf16Char`s that are not ASCII never compare equal.
impl PartialEq<AsciiChar> for Utf16Char {
    #[inline]
    fn eq(&self,  ascii: &AsciiChar) -> bool {
        self.units[0] == *ascii as u16
    }
}
#[cfg(feature = "ascii")]
/// `Utf16Char`s that are not ASCII never compare equal.
impl PartialEq<Utf16Char> for AsciiChar {
    #[inline]
    fn eq(&self,  u16c: &Utf16Char) -> bool {
        *self as u16 == u16c.units[0]
    }
}
#[cfg(feature = "ascii")]
/// `Utf16Char`s that are not ASCII always compare greater.
impl PartialOrd<AsciiChar> for Utf16Char {
    #[inline]
    fn partial_cmp(&self,  ascii: &AsciiChar) -> Option<Ordering> {
        self.units[0].partial_cmp(&(*ascii as u16))
    }
}
#[cfg(feature = "ascii")]
/// `Utf16Char`s that are not ASCII always compare greater.
impl PartialOrd<Utf16Char> for AsciiChar {
    #[inline]
    fn partial_cmp(&self,  u16c: &Utf16Char) -> Option<Ordering> {
        (*self as u16).partial_cmp(&u16c.units[0])
    }
}


  ///////////////////////////////////////////////////////
 //pub impls that should be together for nicer rustdoc//
///////////////////////////////////////////////////////
impl Utf16Char {
    /// Create an `Utf16Char` from the first codepoint in a string slice,
    /// converting from UTF-8 to UTF-16.
    ///
    /// The returned `usize` is the number of UTF-8 bytes used from the str,
    /// and not the number of UTF-16 units.
    ///
    /// Returns an error if the `str` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use encode_unicode::Utf16Char;
    ///
    /// assert_eq!(Utf16Char::from_str_start("a"), Ok((Utf16Char::from('a'),1)));
    /// assert_eq!(Utf16Char::from_str_start("ab"), Ok((Utf16Char::from('a'),1)));
    /// assert_eq!(Utf16Char::from_str_start("🂠 "), Ok((Utf16Char::from('🂠'),4)));
    /// assert_eq!(Utf16Char::from_str_start("é"), Ok((Utf16Char::from('e'),1)));// 'e'+u301 combining mark
    /// assert!(Utf16Char::from_str_start("").is_err());
    /// ```
    pub fn from_str_start(s: &str) -> Result<(Self,usize), EmptyStrError> {
        if s.is_empty() {
            return Err(EmptyStrError);
        }
        let b = s.as_bytes();
        // Read the last byte first to reduce the number of unnecesary length checks.
        match b[0] {
            0...127 => {// 1 byte => 1 unit
                let unit = b[0] as u16;// 0b0000_0000_0xxx_xxxx
                Ok((Utf16Char{ units: [unit, 0] }, 1))
            },
            0b1000_0000...0b1101_1111 => {// 2 bytes => 1 unit
                let unit = (((b[1] & 0x3f) as u16) << 0) // 0b0000_0000_00xx_xxxx
                         | (((b[0] & 0x1f) as u16) << 6);// 0b0000_0xxx_xx00_0000
                Ok((Utf16Char{ units: [unit, 0] }, 2))
            },
            0b1110_0000...0b1110_1111 => {// 3 bytes => 1 unit
                let unit = (((b[2] & 0x3f) as u16) <<  0) // 0b0000_0000_00xx_xxxx
                         | (((b[1] & 0x3f) as u16) <<  6) // 0b0000_xxxx_xx00_0000
                         | (((b[0] & 0x0f) as u16) << 12);// 0bxxxx_0000_0000_0000
                Ok((Utf16Char{ units: [unit, 0] }, 3))
            },
            _ => {// 4 bytes => 2 units
                let second = 0xdc00                        // 0b1101_1100_0000_0000
                           | (((b[3] & 0x3f) as u16) << 0) // 0b0000_0000_00xx_xxxx
                           | (((b[2] & 0x0f) as u16) << 6);// 0b0000_00xx_xx00_0000
                let first = 0xd800-(0x01_00_00u32>>10) as u16// 0b1101_0111_1100_0000
                          + (((b[2] & 0x30) as u16) >> 4)    // 0b0000_0000_0000_00xx
                          + (((b[1] & 0x3f) as u16) << 2)    // 0b0000_0000_xxxx_xx00
                          + (((b[0] & 0x07) as u16) << 8);   // 0b0000_0xxx_0000_0000
                Ok((Utf16Char{ units: [first, second] }, 4))
            }
        }
    }
    /// Validate and store the first UTF-16 codepoint in the slice.
    /// Also return how many units were needed.
    pub fn from_slice_start(src: &[u16]) -> Result<(Self,usize), InvalidUtf16Slice> {
        char::from_utf16_slice_start(src).map(|(_,len)| {
            let second = if len==2 {src[1]} else {0};
            (Utf16Char{ units: [src[0], second] }, len)
        })
    }
    /// Store the first UTF-16 codepoint of the slice.
    ///
    /// # Safety
    /// The slice must be non-empty and start with a valid UTF-16 codepoint.  
    /// The length of the slice is never checked.
    pub unsafe fn from_slice_start_unchecked(src: &[u16]) -> (Self,usize) {
        let first = *src.get_unchecked(0);
        if first.is_utf16_leading_surrogate() {
            (Utf16Char{ units: [first, *src.get_unchecked(1)] }, 2)
        } else {
            (Utf16Char{ units: [first, 0] }, 1)
        }
    }
    /// Validate and store a UTF-16 pair as returned from `char.to_utf16_tuple()`.
    pub fn from_tuple(utf16: (u16,Option<u16>)) -> Result<Self,InvalidUtf16Tuple> {
        unsafe {char::from_utf16_tuple(utf16).map(|_|
            Self::from_tuple_unchecked(utf16)
        )}
    }
    /// Create an `Utf16Char` from a tuple as returned from `char.to_utf16_tuple()`.
    ///
    /// # Safety
    ///
    /// The units must represent a single valid codepoint.
    pub unsafe fn from_tuple_unchecked(utf16: (u16,Option<u16>)) -> Self {
        Utf16Char{ units: [utf16.0, utf16.1.unwrap_or(0)] }
    }
    /// Create an `Utf16Char` from a single unit.
    ///
    /// The unit must be a codepoint in the basic multilingual plane,
    /// or, not a part of a surrogate pair.
    ///
    /// # Errors
    ///
    /// Returns `NonBMPError` if the unit is in the range `0xd800..0xe000`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use encode_unicode::Utf16Char;
    /// assert_eq!(Utf16Char::from_bmp(0x40).unwrap(), '@');
    /// assert_eq!(Utf16Char::from_bmp('ø' as u16).unwrap(), 'ø');
    /// assert!(Utf16Char::from_bmp(0xdddd).is_err());
    /// ```
    pub fn from_bmp(bmp_codepoint: u16) -> Result<Self,NonBMPError> {
        if bmp_codepoint & 0xf800 != 0xd800 {
            Ok(Utf16Char{ units: [bmp_codepoint, 0] })
        } else {
            Err(NonBMPError)
        }
    }
    /// Create an `Utf16Char` from a single unit without checking that it's a
    /// valid codepoint on its own.
    ///
    /// # Safety
    ///
    /// The unit must be less than 0xd800 or greater than 0xdfff codepoint in the basic multilingual plane,
    /// or, not a surrogate.
    #[inline]
    pub unsafe fn from_bmp_unchecked(bmp_codepoint: u16) -> Self {
        Utf16Char{ units: [bmp_codepoint, 0] }
    }

    /// The number of units this character is made up of.
    ///
    /// Is either 1 or 2 and identical to `.as_char().len_utf16()`
    /// or `.as_ref().len()`.
    #[inline]
    pub fn len(self) -> usize {
        1 + (self.units[1] as usize >> 15)
    }
    // There is no `.is_emty()` because it would always return false.

    /// Checks that the codepoint is an ASCII character.
    #[inline]
    pub fn is_ascii(&self) -> bool {
        self.units[0] <= 127
    }
    /// Checks that two characters are an ASCII case-insensitive match.
    ///
    /// Is equivalent to `a.to_ascii_lowercase() == b.to_ascii_lowercase()`.
    #[cfg(feature="std")]
    pub fn eq_ignore_ascii_case(&self,  other: &Self) -> bool {
        self.to_ascii_lowercase() == other.to_ascii_lowercase()
    }
    /// Converts the character to its ASCII upper case equivalent.
    ///
    /// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z',
    /// but non-ASCII letters are unchanged.
    #[cfg(feature="std")]
    pub fn to_ascii_uppercase(&self) -> Self {
        let n = self.units[0].wrapping_sub(b'a' as u16);
        if n < 26 {Utf16Char{ units: [n+b'A' as u16, 0] }}
        else      {*self}
    }
    /// Converts the character to its ASCII lower case equivalent.
    ///
    /// ASCII letters 'A' to 'Z' are mapped to 'a' to 'z',
    /// but non-ASCII letters are unchanged.
    #[cfg(feature="std")]
    pub fn to_ascii_lowercase(&self) -> Self {
        let n = self.units[0].wrapping_sub(b'A' as u16);
        if n < 26 {Utf16Char{ units: [n+b'a' as u16, 0] }}
        else      {*self}
    }
    /// Converts the character to its ASCII upper case equivalent in-place.
    ///
    /// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z',
    /// but non-ASCII letters are unchanged.
    #[cfg(feature="std")]
    pub fn make_ascii_uppercase(&mut self) {
        *self = self.to_ascii_uppercase()
    }
    /// Converts the character to its ASCII lower case equivalent in-place.
    ///
    /// ASCII letters 'A' to 'Z' are mapped to 'a' to 'z',
    /// but non-ASCII letters are unchanged.
    #[cfg(feature="std")]
    pub fn make_ascii_lowercase(&mut self) {
        *self = self.to_ascii_lowercase();
    }

    /// Convert from UTF-16 to UTF-32
    pub fn to_char(self) -> char {
        self.into()
    }
    /// Write the internal representation to a slice,
    /// and then returns the number of `u16`s written.
    ///
    /// # Panics
    /// Will panic the buffer is too small;
    /// You can get the required length from `.len()`,
    /// but a buffer of length two is always large enough.
    pub fn to_slice(self,  dst: &mut[u16]) -> usize {
        // Write the last unit first to avoid repeated length checks.
        let extra = self.units[1] as usize >> 15;
        match dst.get_mut(extra) {
            Some(first) => *first = self.units[extra],
            None => panic!("The provided buffer is too small.")
        }
        if extra != 0 {dst[0] = self.units[0];}
        extra+1
    }
    /// The second `u16` is used for surrogate pairs.
    #[inline]
    pub fn to_tuple(self) -> (u16,Option<u16>) {
        (self.units[0],  if self.units[1]==0 {None} else {Some(self.units[1])})
    }
}
