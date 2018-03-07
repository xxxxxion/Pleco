//! Contains the `MoveList` & `ScoreMoveList` structures, akin to a `Vec<BitMove>` but faster for
//! our purposes.
//!
//! A [`MoveList`] structure is guaranteed to be exactly 512 bytes long, containing a maximum of 256
//! moves. This number was chosen as no possible chess position has been found to contain more than
//! 232 possible moves.
//!
//! This structure is intended to mainly be used for generation of moves for a certain position. If
//! you need to a more versatile collection of moves to manipulate, considering using a `Vec<BitMove>`
//! instead.
//!
//! The [`ScoreMoveList`] is practically the same as the [`MoveList`], but it allows for each move to
//! have a score attached to it as well.
//!
//! [`MoveList`]: struct.MoveList.html
//! [`ScoreMoveList`]: struct.MoveList.html

use std::slice;
use std::mem::transmute;
use std::ops::{Deref,DerefMut,Index,IndexMut};
use std::iter::{Iterator,IntoIterator,FusedIterator,TrustedLen,ExactSizeIterator,FromIterator};

#[allow(unused_imports)]
#[cfg(all(any(target_arch = "x86_64", target_arch = "x86"), target_feature = "avx2"))]
use std::simd::u16x32;


use super::piece_move::{BitMove, ScoringMove};
use super::bitboard::BitBoard;
use super::sq::SQ;

pub trait MVPushable: Sized + IndexMut<usize> + Index<usize> + DerefMut {

    /// Adds a `BitMove` to the end of the list.
    ///
    /// # Safety
    ///
    /// If pushing to the list when at capacity, does nothing.
    fn push_mv(&mut self, mv: BitMove);

    /// Adds a `BitMove` to the end of the list.
    ///
    /// # Safety
    ///
    /// Undefined behavior if pushing to the list when `MoveList::len() = 256`.
    unsafe fn unchecked_push_mv(&mut self, mv: BitMove);

    /// Set the length of the list.
    ///
    /// # Safety
    ///
    /// Unsafe due to overwriting the length of the list
    unsafe fn unchecked_set_len(&mut self, len: usize);


    /// Return a pointer to the first (0-th index) element in the list
    ///
    /// # Safety
    ///
    /// Unsafe due to allow modification of elements possibly not inside the length.
    unsafe fn list_ptr(&mut self) -> *mut Self::Output;

    /// Return a pointer to the element next to the last element in the list
    ///
    /// # Safety
    ///
    /// Unsafe due to allow modification of elements possibly not inside the length.
    unsafe fn over_bounds_ptr(&mut self) -> *mut Self::Output;

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx2"))]
    #[doc(hidden)]
    unsafe fn avx_append(ptr: *mut Self::Output, src: SQ, dst: &mut BitBoard, flags: u16) -> *mut Self::Output;
}

const MAX_MOVES: usize = 256;

/// This is the list of possible moves for a current position. Think of it alike a faster
/// version of `Vec<BitMove>`, as all the data is stored in the Stack rather than the Heap.
pub struct MoveList {
    inner: [BitMove; 256],
    len: usize,
}

impl Default for MoveList {
    #[inline]
    fn default() -> Self {
        MoveList {
            inner: [BitMove::null(); 256],
            len: 0,
        }
    }
}

impl From<Vec<BitMove>> for MoveList {
    fn from(vec: Vec<BitMove>) -> Self {
        let mut list = MoveList::default();
        vec.iter().for_each(|m| list.push(*m));
        list
    }
}


impl From<ScoringMoveList> for MoveList {
    fn from(sc_list: ScoringMoveList) -> Self {
        let mut mv_list = MoveList::default();
        sc_list.iter().for_each(|m| mv_list.push(m.bitmove()));
        mv_list
    }
}

impl Into<Vec<BitMove>> for MoveList {
    #[inline]
    fn into(self) -> Vec<BitMove> {
        self.vec()
    }
}

impl MoveList {
    /// Adds a `BitMove` to the end of the list.
    ///
    /// # Safety
    ///
    /// If pushing to the list when at capacity, does nothing.
    #[inline(always)]
    pub fn push(&mut self, mv: BitMove) {
        self.push_mv(mv);
    }

    /// Returns true if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{BitMove,MoveList};
    ///
    /// let mut list = MoveList::default();
    /// assert!(list.is_empty());
    /// ```
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Creates a `Vec<BitMove>` from this `MoveList`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{BitMove,MoveList};
    ///
    /// let mut list = MoveList::default();
    /// list.push(BitMove::null());
    ///
    /// let vec: Vec<BitMove> = list.vec();
    /// ```
    pub fn vec(&self) -> Vec<BitMove> {
        let mut vec = Vec::with_capacity(self.len);
        for mov in self.iter() {
            vec.push(*mov);
        }
        assert_eq!(vec.len(),self.len);
        vec
    }

    /// Returns the number of moves inside the list.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{BitMove,MoveList};
    ///
    /// let mut list = MoveList::default();
    /// list.push(BitMove::null());
    /// list.push(BitMove::null());
    /// list.push(BitMove::null());
    /// assert_eq!(list.len(), 3);
    /// ```
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the `MoveList` as a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{BitMove,MoveList};
    ///
    /// let mut list = MoveList::default();
    /// list.push(BitMove::null());
    ///
    /// let slice: &[BitMove] = list.as_slice();
    /// ```
    #[inline(always)]
    pub fn as_slice(&self) -> &[BitMove] {
        self
    }

    /// Returns the `MoveList` as a mutable slice.
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [BitMove] {
        self
    }
}


impl Deref for MoveList {
    type Target = [BitMove];

    #[inline]
    fn deref(&self) -> &[BitMove] {
        unsafe {
            let p = self.inner.as_ptr();
            slice::from_raw_parts(p, self.len)
        }
    }
}

impl DerefMut for MoveList {
    #[inline]
    fn deref_mut(&mut self) -> &mut [BitMove] {
        unsafe {
            let ptr = self.inner.as_mut_ptr();
            slice::from_raw_parts_mut(ptr, self.len)
        }
    }
}

impl Index<usize> for MoveList {
    type Output = BitMove;

    #[inline(always)]
    fn index(&self, index: usize) -> &BitMove {
        &(**self)[index]
    }
}

impl IndexMut<usize> for MoveList {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut BitMove {
        &mut (**self)[index]
    }
}


impl MVPushable for MoveList {
    #[inline(always)]
    fn push_mv(&mut self, mv: BitMove) {
        if self.len() < MAX_MOVES {
            unsafe{ self.unchecked_push_mv(mv) }
        }
    }

    #[inline(always)]
    unsafe fn unchecked_push_mv(&mut self, mv: BitMove) {
        let end = self.inner.get_unchecked_mut(self.len);
        *end = mv;
        self.len += 1;
    }

    #[inline(always)]
    unsafe fn unchecked_set_len(&mut self, len: usize) {
        self.len = len
    }

    #[inline(always)]
    unsafe fn list_ptr(&mut self) -> *mut BitMove {
        self.as_mut_ptr()
    }

    #[inline(always)]
    unsafe fn over_bounds_ptr(&mut self) -> *mut BitMove {
        self.as_mut_ptr().add(self.len)
    }


    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx2"))]
    #[doc(hidden)]
    #[inline(always)]
    unsafe fn avx_append(mut ptr: *mut BitMove, src: SQ, dst: &mut BitBoard, flags: u16) -> *mut BitMove {
        let mut i = 0;
        let mut v = u16x32::splat(0);

        while let Some(dst_sq) = dst.pop_some_lsb() {
            v = v.replace_unchecked(i, dst_sq.0 as u16);
            i += 1;
        }

        if i != 0 {
            v <<= 6;
            v |= u16x32::splat(flags << 12 | src.0 as u16);
            for x in 0..i {
                *ptr = transmute(v.extract_unchecked(x));
                ptr = ptr.add(1);
            }
        }

        ptr
    }
}

pub struct MoveIter<'a> {
    movelist: &'a MoveList,
    idx: usize
}

impl<'a> Iterator for MoveIter<'a> {
    type Item = BitMove;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.movelist.len {
            None
        } else {
            unsafe {
                let m = *self.movelist.inner.get_unchecked(self.idx);
                self.idx += 1;
                Some(m)
            }

        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.movelist.len - self.idx, Some(self.movelist.len - self.idx))
    }
}

impl<'a> IntoIterator for &'a MoveList {
    type Item = BitMove;
    type IntoIter = MoveIter<'a>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        MoveIter {
            movelist: self,
            idx: 0,
        }
    }
}

impl<'a> ExactSizeIterator for MoveIter<'a> {}

impl<'a> FusedIterator for MoveIter<'a> {}

unsafe impl<'a> TrustedLen for MoveIter<'a> {}

// Iterator for the `MoveList`.
pub struct MoveIntoIter {
    movelist: MoveList,
    idx: usize
}

impl Iterator for MoveIntoIter {
    type Item = BitMove;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.movelist.len {
            None
        } else {
            unsafe {
                let m = *self.movelist.inner.get_unchecked(self.idx);
                self.idx += 1;
                Some(m)
            }

        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.movelist.len - self.idx, Some(self.movelist.len - self.idx))
    }
}

impl IntoIterator for MoveList {
    type Item = BitMove;
    type IntoIter = MoveIntoIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        MoveIntoIter {
            movelist: self,
            idx: 0,
        }
    }
}

impl FromIterator<BitMove> for MoveList {
    fn from_iter<T: IntoIterator<Item=BitMove>>(iter: T) -> Self {
        let mut list = MoveList::default();
        for i in iter {
            list.push(i);
        }
        list
    }
}

impl ExactSizeIterator for MoveIntoIter {}

impl FusedIterator for MoveIntoIter {}

unsafe impl TrustedLen for MoveIntoIter {}


/// This is similar to a `MoveList`, but also keeps the scores for each move as well.
pub struct ScoringMoveList {
    inner: [ScoringMove; 256],
    len: usize,
}

impl Default for ScoringMoveList {
    #[inline]
    fn default() -> Self {
        ScoringMoveList {
            inner: [ScoringMove::default(); 256],
            len: 0,
        }
    }
}

impl From<Vec<BitMove>> for ScoringMoveList {
    fn from(vec: Vec<BitMove>) -> Self {
        let mut list = ScoringMoveList::default();
        vec.iter().for_each(|m| list.push(*m));
        list
    }
}

impl From<MoveList> for ScoringMoveList {
    fn from(mv_list: MoveList) -> Self {
        let mut sc_list = ScoringMoveList::default();
        mv_list.iter().for_each(|m| sc_list.push(*m));
        sc_list
    }
}

impl Into<Vec<ScoringMove>> for ScoringMoveList {
    #[inline]
    fn into(self) -> Vec<ScoringMove> {
        self.vec()
    }
}

impl ScoringMoveList {
    /// Adds a `BitMove` to the end of the list.
    ///
    /// # Safety
    ///
    /// If pushing to the list when at capacity, does nothing.
    #[inline(always)]
    pub fn push(&mut self, mov: BitMove) {
        self.push_mv(mov)
    }

    /// Returns true if empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Creates a vector from this `MoveList`.
    pub fn vec(&self) -> Vec<ScoringMove> {
        let mut vec = Vec::with_capacity(self.len);
        for pair in self.iter() {
            vec.push(*pair);
        }
        assert_eq!(vec.len(),self.len);
        vec
    }

    /// Returns the number of moves inside the list.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the `ScoringMoveList` as a slice.
    #[inline(always)]
    pub fn as_slice(&self) -> &[ScoringMove] {
        self
    }

    /// Returns the `ScoringMoveList` as a mutable slice.
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [ScoringMove] {
        self
    }
}


impl Deref for ScoringMoveList {
    type Target = [ScoringMove];

    #[inline]
    fn deref(&self) -> &[ScoringMove] {
        unsafe {
            let p = self.inner.as_ptr();
            slice::from_raw_parts(p, self.len)
        }
    }
}

impl DerefMut for ScoringMoveList {
    #[inline]
    fn deref_mut(&mut self) -> &mut [ScoringMove] {
        unsafe {
            let ptr = self.inner.as_mut_ptr();
            slice::from_raw_parts_mut(ptr, self.len)
        }
    }
}

impl Index<usize> for ScoringMoveList {
    type Output = ScoringMove;

    #[inline(always)]
    fn index(&self, index: usize) -> &ScoringMove {
        &(**self)[index]
    }
}

impl IndexMut<usize> for ScoringMoveList {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut ScoringMove {
        &mut (**self)[index]
    }
}

impl MVPushable for ScoringMoveList {
    #[inline(always)]
    fn push_mv(&mut self, mv: BitMove) {
        if self.len() < MAX_MOVES {
            unsafe{ self.unchecked_push_mv(mv) }
        }
    }

    #[inline(always)]
    unsafe fn unchecked_push_mv(&mut self, mv: BitMove) {
        let end = self.inner.get_unchecked_mut(self.len);
        *end = ScoringMove::new(mv);
        self.len += 1;
    }


    #[inline(always)]
    unsafe fn unchecked_set_len(&mut self, len: usize) {
        self.len = len
    }

    #[inline(always)]
    unsafe fn list_ptr(&mut self) -> *mut ScoringMove {
        self.as_mut_ptr()
    }

    #[inline(always)]
    unsafe fn over_bounds_ptr(&mut self) -> *mut ScoringMove {
        self.as_mut_ptr().add(self.len)
    }

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx2"))]
    #[doc(hidden)]
    #[inline(always)]
    unsafe fn avx_append(mut ptr: *mut ScoringMove, src: SQ, dst: &mut BitBoard, flags: u16) -> *mut ScoringMove {
        let mut i = 0;
        let mut v = u16x32::splat(0);
        while let Some(dst_sq) = dst.pop_some_lsb() {
            v = v.replace_unchecked(i, dst_sq.0 as u16);
            i += 1;
        }

        if i != 0 {
            v *= u16x32::splat(64);
            v |= u16x32::splat(flags << 12 | src.0 as u16);

            for x in 0..i {
                (*ptr).bit_move = transmute(v.extract_unchecked(x));
                ptr = ptr.add(1);
            }
        }
        ptr
    }
}

pub struct ScoreMoveIter<'a> {
    movelist: &'a ScoringMoveList,
    idx: usize
}

impl<'a> Iterator for ScoreMoveIter<'a> {
    type Item = ScoringMove;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.movelist.len {
            None
        } else {
            unsafe {
                let m = *self.movelist.inner.get_unchecked(self.idx);
                self.idx += 1;
                Some(m)
            }

        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.movelist.len - self.idx, Some(self.movelist.len - self.idx))
    }
}

impl<'a> IntoIterator for &'a ScoringMoveList {
    type Item = ScoringMove;
    type IntoIter = ScoreMoveIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ScoreMoveIter {
            movelist: self,
            idx: 0,
        }
    }
}

impl<'a> ExactSizeIterator for ScoreMoveIter<'a> {}

impl<'a> FusedIterator for ScoreMoveIter<'a> {}

unsafe impl<'a> TrustedLen for ScoreMoveIter<'a> {}

// Iterator for the `ScoringMoveList`.
pub struct ScoreMoveIntoIter {
    movelist: ScoringMoveList,
    idx: usize
}

impl Iterator for ScoreMoveIntoIter {
    type Item = ScoringMove;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.movelist.len {
            None
        } else {
            unsafe {
                let m = *self.movelist.inner.get_unchecked(self.idx);
                self.idx += 1;
                Some(m)
            }

        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.movelist.len - self.idx, Some(self.movelist.len - self.idx))
    }
}

impl IntoIterator for ScoringMoveList {
    type Item = ScoringMove;
    type IntoIter = ScoreMoveIntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ScoreMoveIntoIter {
            movelist: self,
            idx: 0,
        }
    }
}

impl FromIterator<BitMove> for ScoringMoveList {
    fn from_iter<T: IntoIterator<Item=BitMove>>(iter: T) -> Self {
        let mut list = ScoringMoveList::default();
        for i in iter {
            list.push(i);
        }
        list
    }
}

impl ExactSizeIterator for ScoreMoveIntoIter {}

impl FusedIterator for ScoreMoveIntoIter {}

unsafe impl TrustedLen for ScoreMoveIntoIter {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_avx2() {
        let mut arr = [BitMove::null(); 16];
        let mut bb = BitBoard::FILE_B;
        unsafe {
            MoveList::avx_append(arr.as_mut_ptr(), SQ::A1, &mut bb, 0b1111);
        }
    }
}