//! `core::iter::Step` usage
//!
//! Tracking issue [42168].
//!
//! Example of impl `Step` here is for `CustomIndex`.
//!
//! `CustomIndex` internally is implemented with `u32`. This emulates the result of [`rustc_index_macros::newtype_index`],
//! which is used for [`rustc_index::IndexVec`].
//!
//! [42168]: https://github.com/rust-lang/rust/issues/42168
//! [`rustc_index_macros::newtype_index`]: https://github.com/rust-lang/rust/blob/5e17a2a91dd7dbefd8b4a1087c2e42257457deeb/compiler/rustc_index_macros/src/lib.rs#L38
//! [`rustc_index::IndexVec`]: https://github.com/rust-lang/rust/blob/5e17a2a91dd7dbefd8b4a1087c2e42257457deeb/compiler/rustc_index/src/vec.rs#L40

#![allow(dead_code)]

use std::{fmt, hash, iter::Step};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "nightly", rustc_layout_scalar_valid_range_end(0xFFFF_FF00))]
#[cfg_attr(feature = "nightly", rustc_pass_by_value)]
struct CustomIndex {
    private_use_as_methods_instead: u32,
}

// shave off 256 indices at the end to allow space for packing these indices into enums
// IMPORTANT: used in #![feature(rustc_layout_scalar_valid_range_end)]
const MAX: u32 = 0xFFFF_FF00;

impl CustomIndex {
    /// Maximum value the index can take, as a `u32`.
    pub const MAX_AS_U32: u32 = MAX;

    /// Maximum value the index can take.
    pub const MAX: Self = Self::from_u32(MAX);

    /// Zero value of the index.
    pub const ZERO: Self = Self::from_u32(0);

    /// Creates a new index from a given `usize`.
    ///
    /// # Panics
    ///
    /// Will panic if `value` exceeds `MAX`.
    #[inline]
    pub const fn from_usize(value: usize) -> Self {
        assert!(value <= (MAX as usize));
        // SAFETY: We just checked that `value <= max`.
        unsafe { Self::from_u32_unchecked(value as u32) }
    }

    /// Creates a new index from a given `u32`.
    ///
    /// # Panics
    ///
    /// Will panic if `value` exceeds `MAX`.
    #[inline]
    pub const fn from_u32(value: u32) -> Self {
        assert!(value <= MAX);
        // SAFETY: We just checked that `value <= max`.
        unsafe { Self::from_u32_unchecked(value) }
    }

    /// Creates a new index from a given `u16`.
    ///
    /// # Panics
    ///
    /// Will panic if `value` exceeds `MAX`.
    #[inline]
    pub const fn from_u16(value: u16) -> Self {
        let value = value as u32;
        assert!(value <= MAX);
        // SAFETY: We just checked that `value <= max`.
        unsafe { Self::from_u32_unchecked(value) }
    }

    /// Creates a new index from a given `u32`.
    ///
    /// # Safety
    ///
    /// The provided value must be less than or equal to the maximum value for the newtype.
    /// Providing a value outside this range is undefined due to layout restrictions.
    ///
    /// Prefer using `from_u32`.
    #[inline]
    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
        Self {
            private_use_as_methods_instead: value,
        }
    }

    /// Extracts the value of this index as a `usize`.
    #[inline]
    pub const fn index(self) -> usize {
        self.as_usize()
    }

    /// Extracts the value of this index as a `u32`.
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.private_use_as_methods_instead
    }

    /// Extracts the value of this index as a `usize`.
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.as_u32() as usize
    }
}

impl std::ops::Add<usize> for CustomIndex {
    type Output = Self;

    #[inline]
    fn add(self, other: usize) -> Self {
        Self::from_usize(self.index() + other)
    }
}

impl Idx for CustomIndex {
    #[inline]
    fn new(idx: usize) -> Self {
        Self::from_usize(idx)
    }

    #[inline]
    fn index(self) -> usize {
        self.as_usize()
    }
}

#[cfg(feature = "nightly")]
impl Step for CustomIndex {
    #[inline]
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        <usize as Step>::steps_between(&start.index(), &end.index())
    }

    #[inline]
    fn forward_checked(start: Self, u: usize) -> Option<Self> {
        Self::index(start).checked_add(u).map(Self::from_usize)
    }

    #[inline]
    fn backward_checked(start: Self, u: usize) -> Option<Self> {
        Self::index(start).checked_sub(u).map(Self::from_usize)
    }
}

impl From<CustomIndex> for u32 {
    #[inline]
    fn from(v: CustomIndex) -> u32 {
        v.as_u32()
    }
}

impl From<CustomIndex> for usize {
    #[inline]
    fn from(v: CustomIndex) -> usize {
        v.as_usize()
    }
}

impl From<usize> for CustomIndex {
    #[inline]
    fn from(value: usize) -> Self {
        Self::from_usize(value)
    }
}

impl From<u32> for CustomIndex {
    #[inline]
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

/// Represents some newtyped `usize` wrapper.
///
/// **This is copy of rustc_index.**
///
/// Purpose: avoid mixing indexes for different bitvector domains.
pub trait Idx: Copy + 'static + Eq + PartialEq + fmt::Debug + hash::Hash {
    fn new(idx: usize) -> Self;

    fn index(self) -> usize;

    #[inline]
    fn increment_by(&mut self, amount: usize) {
        *self = self.plus(amount);
    }

    #[inline]
    #[must_use = "Use `increment_by` if you wanted to update the index in-place"]
    fn plus(self, amount: usize) -> Self {
        Self::new(self.index() + amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward() {
        let initial = CustomIndex::new(0);
        assert_eq!(CustomIndex::forward(initial, 1), 1u32.into());
    }

    #[test]
    #[should_panic]
    fn forward_overflow() {
        let initial = CustomIndex::new(0);
        CustomIndex::forward(initial, usize::MAX);
    }

    #[test]
    fn backward() {
        let initial = CustomIndex::new(100);
        assert_eq!(CustomIndex::backward(initial, 1), 99u32.into());
    }

    #[test]
    #[should_panic]
    fn backward_overflow() {
        let initial = CustomIndex::new(1);
        CustomIndex::backward(initial, 2);
    }
}
