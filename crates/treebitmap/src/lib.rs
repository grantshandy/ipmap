// Copyright 2016 Hroi Sigurdsson
//
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! # Fast IP lookup table for IPv4/IPv6 prefixes
//!
//! This crate provides a datastructure for fast IP address lookups.
//! It aims at fast lookup times, and a small memory footprint.
//! A full IPv4 BGP table of more than 600k entries fits in less than 5 MB. A
//! full IPv6 BGP table of more than 25k entries fits in less than 1 MB.
//!
//! Longest match lookups on full BGP IP tables take on the order of 100ns.
//!
//! The internal datastructure is based on the Tree-bitmap algorithm described
//! by W. Eatherton, Z. Dittia, G. Varghes.

use std::marker::PhantomData;

mod tree_bitmap;
use tree_bitmap::{Matches, TreeBitmap};

mod address;
pub use address::Address;

/// A fast, compressed IP lookup table.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IpLookupTable<A, T> {
    inner: TreeBitmap<T>,
    _addrtype: PhantomData<A>,
}

impl<A, T: Copy + Clone + Default> IpLookupTable<A, T>
where
    A: Address,
{
    /// Initialize an empty lookup table with no preallocation.
    pub fn new() -> Self {
        IpLookupTable {
            inner: TreeBitmap::new(),
            _addrtype: PhantomData,
        }
    }

    /// Initialize an empty lookup table with pre-allocated buffers.
    pub fn with_capacity(n: usize) -> Self {
        IpLookupTable {
            inner: TreeBitmap::with_capacity(n),
            _addrtype: PhantomData,
        }
    }

    /// Return the bytes used by nodes and results.
    pub fn mem_usage(&self) -> (usize, usize) {
        self.inner.mem_usage()
    }

    /// Return number of items inside table.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Return `true` if no item is inside table.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Insert a value for the prefix designated by ip and masklen. If prefix
    /// existed previously, the old value is returned.
    ///
    /// # Panics
    ///
    /// Panics if prefix has bits set to the right of mask.
    ///
    /// # Examples
    ///
    /// ```
    /// use treebitmap::IpLookupTable;
    /// use std::net::Ipv6Addr;
    ///
    /// let mut table = IpLookupTable::new();
    /// let prefix = Ipv6Addr::new(0x2001, 0xdb8, 0xdead, 0xbeef, 0, 0, 0, 0);
    /// let masklen = 32;
    ///
    /// assert_eq!(table.insert(prefix, masklen, "foo"), None);
    /// // Insert duplicate
    /// assert_eq!(table.insert(prefix, masklen, "bar"), Some("foo"));
    /// ```
    pub fn insert(&mut self, ip: A, masklen: u32, value: T) -> Option<T> {
        self.inner.insert(ip.nibbles().as_ref(), masklen, value)
    }

    /// Remove an entry from the lookup table. If the prefix existed previously,
    /// the value is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use treebitmap::IpLookupTable;
    /// use std::net::Ipv6Addr;
    ///
    /// let mut table = IpLookupTable::new();
    /// let prefix = Ipv6Addr::new(0x2001, 0xdb8, 0xdead, 0xbeef, 0, 0, 0, 0);
    /// let masklen = 32;
    /// table.insert(prefix, masklen, "foo");
    ///
    /// assert_eq!(table.remove(prefix, masklen), Some("foo"));
    /// // Remove non-existant
    /// assert_eq!(table.remove(prefix, masklen), None);
    /// ```
    pub fn remove(&mut self, ip: A, masklen: u32) -> Option<T> {
        self.inner.remove(ip.nibbles().as_ref(), masklen)
    }

    /// Perform exact match lookup of `ip`/`masklen` and return the
    /// value.
    ///
    /// # Panics
    ///
    /// Panics if prefix has bits set to the right of mask.
    ///
    /// # Examples
    ///
    /// ```
    /// use treebitmap::IpLookupTable;
    /// use std::net::Ipv6Addr;
    ///
    /// let mut table = IpLookupTable::new();
    /// let prefix = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0);
    /// let masklen = 32;
    /// table.insert(prefix, masklen, "foo");
    ///
    /// assert_eq!(table.exact_match(prefix, masklen), Some(&"foo"));
    /// // differing mask
    /// assert_eq!(table.exact_match(prefix, 48), None);
    /// ```
    pub fn exact_match(&self, ip: A, masklen: u32) -> Option<&T> {
        self.inner.exact_match(ip.nibbles().as_ref(), masklen)
    }

    /// Perform exact match lookup of `ip`/`masklen` and return the
    /// value as mutable.
    ///
    /// # Examples
    ///
    /// ```
    /// use treebitmap::IpLookupTable;
    /// use std::net::Ipv6Addr;
    ///
    /// let mut table = IpLookupTable::new();
    /// let prefix = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0);
    /// let masklen = 32;
    /// table.insert(prefix, masklen, "foo");
    ///
    /// assert_eq!(table.exact_match(prefix, masklen), Some(&"foo"));
    /// // Mutate value
    /// if let Some(value) = table.exact_match_mut(prefix, masklen) {
    ///     *value = &"bar";
    /// }
    /// // Get new value
    /// assert_eq!(table.exact_match(prefix, masklen), Some(&"bar"));
    /// ```
    pub fn exact_match_mut(&mut self, ip: A, masklen: u32) -> Option<&mut T> {
        self.inner.exact_match_mut(ip.nibbles().as_ref(), masklen)
    }

    /// Perform longest match lookup of `ip` and return the best matching
    /// prefix, designated by ip, masklen, along with its value.
    ///
    /// # Example
    ///
    /// ```
    /// use treebitmap::IpLookupTable;
    /// use std::net::Ipv6Addr;
    ///
    /// let mut table = IpLookupTable::new();
    /// let less_specific = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0);
    /// let more_specific = Ipv6Addr::new(0x2001, 0xdb8, 0xdead, 0, 0, 0, 0, 0);
    /// table.insert(less_specific, 32, "foo");
    /// table.insert(more_specific, 48, "bar");
    ///
    /// let lookupip = Ipv6Addr::new(0x2001, 0xdb8, 0xdead, 0xbeef,
    ///                              0xcafe, 0xbabe, 0, 1);
    /// let result = table.longest_match(lookupip);
    /// assert_eq!(result, Some((more_specific, 48, &"bar")));
    ///
    /// let lookupip = Ipv6Addr::new(0x2001, 0xdb8, 0xcafe, 0xf00,
    ///                              0xf00, 0xf00, 0, 1);
    /// let result = table.longest_match(lookupip);
    /// assert_eq!(result, Some((less_specific, 32, &"foo")));
    /// ```
    pub fn longest_match(&self, ip: A) -> Option<(A, u32, &T)> {
        self.inner
            .longest_match(ip.nibbles().as_ref())
            .map(|(bits_matched, value)| (ip.mask(bits_matched), bits_matched, value))
    }

    /// Perform longest match lookup of `ip` and return the best matching
    /// prefix, designated by ip, masklen, along with its value as mutable.
    ///
    /// # Example
    ///
    /// ```
    /// use treebitmap::IpLookupTable;
    /// use std::net::Ipv6Addr;
    ///
    /// let mut table = IpLookupTable::new();
    /// let less_specific = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0);
    /// let more_specific = Ipv6Addr::new(0x2001, 0xdb8, 0xdead, 0, 0, 0, 0, 0);
    /// table.insert(less_specific, 32, "foo");
    /// table.insert(more_specific, 48, "bar");
    ///
    /// let lookupip = Ipv6Addr::new(0x2001, 0xdb8, 0xdead, 0xbeef,
    ///                              0xcafe, 0xbabe, 0, 1);
    /// if let Some((_, _, value)) = table.longest_match_mut(lookupip) {
    ///     assert_eq!(value, &"bar");
    ///     *value = &"foo";
    /// }
    ///
    /// let result = table.longest_match(lookupip);
    /// assert_eq!(result, Some((more_specific, 48, &"foo")));
    /// ```
    pub fn longest_match_mut(&mut self, ip: A) -> Option<(A, u32, &mut T)> {
        self.inner
            .longest_match_mut(ip.nibbles().as_ref())
            .map(|(bits_matched, value)| (ip.mask(bits_matched), bits_matched, value))
    }

    /// Perform match lookup of `ip` and return the all matching
    /// prefixes, designated by ip, masklen, along with its value.
    ///
    /// # Example
    ///
    /// ```
    /// use treebitmap::IpLookupTable;
    /// use std::net::Ipv6Addr;
    ///
    /// let mut table = IpLookupTable::new();
    /// let less_specific = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0);
    /// let more_specific = Ipv6Addr::new(0x2001, 0xdb8, 0xdead, 0, 0, 0, 0, 0);
    /// table.insert(less_specific, 32, "foo");
    /// table.insert(more_specific, 48, "bar");
    ///
    /// let lookupip = Ipv6Addr::new(0x2001, 0xdb8, 0xdead, 0xbeef,
    ///                              0xcafe, 0xbabe, 0, 1);
    /// let matches = table.matches(lookupip);
    /// assert_eq!(matches.count(), 2);
    ///
    /// let lookupip = Ipv6Addr::new(0x2001, 0xdb8, 0xcafe, 0xf00,
    ///                              0xf00, 0xf00, 0, 1);
    /// let matches = table.matches(lookupip);
    /// assert_eq!(matches.count(), 1);
    /// ```
    pub fn matches(&self, ip: A) -> impl Iterator<Item = (A, u32, &T)> {
        let nibbles = ip.nibbles();
        let inner = self.inner.matches(nibbles.as_ref());

        MatchesIter { inner, ip }
    }

    /// Perform match lookup of `ip` and return the all matching
    /// prefixes, designated by ip, masklen, along with its mutable value.
    pub fn matches_mut(&mut self, ip: A) -> impl Iterator<Item = (A, u32, &mut T)> {
        self.inner
            .matches_mut(ip.nibbles().as_ref())
            .map(move |(bits_matched, value)| (ip.mask(bits_matched), bits_matched, value))
    }

    /// Returns iterator over prefixes and values.
    ///
    /// # Examples
    ///
    /// ```
    /// use treebitmap::IpLookupTable;
    /// use std::net::Ipv6Addr;
    ///
    /// let mut table = IpLookupTable::new();
    /// let less_specific = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0);
    /// let more_specific = Ipv6Addr::new(0x2001, 0xdb8, 0xdead, 0, 0, 0, 0, 0);
    /// table.insert(less_specific, 32, "foo");
    /// table.insert(more_specific, 48, "bar");
    ///
    /// let mut iter = table.iter();
    /// assert_eq!(iter.next(), Some((less_specific, 32, &"foo")));
    /// assert_eq!(iter.next(), Some((more_specific, 48, &"bar")));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<'_, A, T> {
        Iter {
            inner: self.inner.iter(),
            _addrtype: PhantomData,
        }
    }

    /// Mutable version of iter().
    ///
    /// # Examples
    ///
    /// ```
    /// use treebitmap::IpLookupTable;
    /// use std::net::Ipv6Addr;
    ///
    /// let x: Ipv6Addr = "2001:db8:100::".parse().unwrap();
    /// let y: Ipv6Addr = "2001:db8:100::".parse().unwrap();
    /// let z: Ipv6Addr = "2001:db8:102::".parse().unwrap();
    /// let mut table = IpLookupTable::new();
    ///
    /// table.insert(x, 48, 1);
    /// table.insert(y, 56, 2);
    /// table.insert(z, 56, 3);
    ///
    /// for (_ip, _mask, val) in table.iter_mut() {
    ///     *val += 10;
    /// }
    ///
    /// assert_eq!(table.exact_match(x, 48), Some(&11));
    /// assert_eq!(table.exact_match(y, 56), Some(&12));
    /// assert_eq!(table.exact_match(z, 56), Some(&13));
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, A, T> {
        IterMut {
            inner: self.inner.iter_mut(),
            _addrtype: PhantomData,
        }
    }
}

impl<A, T> Default for IpLookupTable<A, T>
where
    T: Clone + Copy + Default,
    A: Address,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Address + Ord, T: Clone + Copy + Ord + Default> PartialEq for IpLookupTable<A, T> {
    fn eq(&self, other: &Self) -> bool {
        let mut self_entries: Vec<(A, u32, &T)> = self.iter().collect();
        let mut other_entries: Vec<(A, u32, &T)> = other.iter().collect();

        if self_entries.len() != other_entries.len() {
            return false;
        }

        self_entries.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.cmp(&b.1))
                .then_with(|| a.2.cmp(b.2))
        });

        other_entries.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.cmp(&b.1))
                .then_with(|| a.2.cmp(b.2))
        });

        self_entries.iter().eq(other_entries.iter())
    }
}

impl<'a, A, T: 'a> Iterator for Iter<'a, A, T>
where
    T: Clone + Copy + Default,
    A: Address,
{
    type Item = (A, u32, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(nibbles, masklen, value)| (Address::from_nibbles(&nibbles[..]), masklen, value))
    }
}

impl<'a, A, T: 'a> Iterator for IterMut<'a, A, T>
where
    T: Clone + Copy + Default,
    A: Address,
{
    type Item = (A, u32, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(nibbles, masklen, value)| (Address::from_nibbles(&nibbles[..]), masklen, value))
    }
}

impl<A, T> Iterator for IntoIter<A, T>
where
    T: Clone + Copy + Default,
    A: Address,
{
    type Item = (A, u32, T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(nibbles, masklen, value)| (Address::from_nibbles(&nibbles[..]), masklen, value))
    }
}

impl<A, T> IntoIterator for IpLookupTable<A, T>
where
    T: Clone + Copy + Default,
    A: Address,
{
    type Item = (A, u32, T);
    type IntoIter = IntoIter<A, T>;

    fn into_iter(self) -> IntoIter<A, T> {
        IntoIter {
            inner: self.inner.into_iter(),
            _addrtype: PhantomData,
        }
    }
}

/// Iterator over prefixes and associated values. The prefixes are returned in
/// "tree"-order.
#[doc(hidden)]
pub struct Iter<'a, A, T: 'a> {
    inner: tree_bitmap::Iter<'a, T>,
    _addrtype: PhantomData<A>,
}

/// Mutable iterator over prefixes and associated values. The prefixes are
/// returned in "tree"-order.
#[doc(hidden)]
pub struct IterMut<'a, A, T: 'a> {
    inner: tree_bitmap::IterMut<'a, T>,
    _addrtype: PhantomData<A>,
}

/// Converts ```IpLookupTable``` into an iterator. The prefixes are returned in
/// "tree"-order.
#[doc(hidden)]
pub struct IntoIter<A, T> {
    inner: tree_bitmap::IntoIter<T>,
    _addrtype: PhantomData<A>,
}

struct MatchesIter<'a, T: Copy + Clone + Default, A> {
    inner: Matches<'a, T>,
    ip: A,
}

impl<'a, T, A> Iterator for MatchesIter<'a, T, A>
where
    T: Copy + Clone + Default,
    A: Address,
{
    type Item = (A, u32, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(bits_matched, value)| (self.ip.mask(bits_matched), bits_matched, value))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[rustfmt::skip]
    #[test]
    fn test_partial_eq() {
        let mut tbl1 = IpLookupTable::<core::net::Ipv4Addr, i32>::new();
        tbl1.insert(core::net::Ipv4Addr::new(10, 0, 0, 1), 17, 1);
        tbl1.insert(core::net::Ipv4Addr::new(172, 16, 0, 1), 17, 2);
        tbl1.insert(core::net::Ipv4Addr::new(192, 168, 1, 1), 17, 3);
        tbl1.insert(core::net::Ipv4Addr::new(8, 8, 8, 8), 17, 4);

        // insertion order shouldn't affect equality
        let mut tbl2 = IpLookupTable::<core::net::Ipv4Addr, i32>::new();
        tbl2.insert(core::net::Ipv4Addr::new(172, 16, 0, 1), 17, 2);
        tbl2.insert(core::net::Ipv4Addr::new(10, 0, 0, 1), 17, 1);
        tbl2.insert(core::net::Ipv4Addr::new(8, 8, 8, 8), 17, 4);
        tbl2.insert(core::net::Ipv4Addr::new(192, 168, 1, 1), 17, 3);
        assert_eq!(tbl1, tbl2);

        // mismatching data
        let mut tbl3 = IpLookupTable::<core::net::Ipv4Addr, i32>::new();
        tbl3.insert(core::net::Ipv4Addr::new(10, 0, 0, 1), 17, 100);
        tbl3.insert(core::net::Ipv4Addr::new(172, 16, 0, 1), 17, 2);
        tbl3.insert(core::net::Ipv4Addr::new(192, 168, 1, 1), 17, 3);
        tbl3.insert(core::net::Ipv4Addr::new(8, 8, 8, 8), 17, 4);
        assert_ne!(tbl1, tbl3);

        // IP missing
        let mut tbl4 = IpLookupTable::<core::net::Ipv4Addr, i32>::new();
        tbl4.insert(core::net::Ipv4Addr::new(10, 0, 0, 1), 17, 1);
        tbl4.insert(core::net::Ipv4Addr::new(172, 16, 0, 1), 17, 2);
        tbl4.insert(core::net::Ipv4Addr::new(192, 168, 1, 1), 17, 3);
        assert_ne!(tbl1, tbl4);

        // Extra IP
        let mut tbl5 = IpLookupTable::<core::net::Ipv4Addr, i32>::new();
        tbl5.insert(core::net::Ipv4Addr::new(10, 0, 0, 1), 17, 1);
        tbl5.insert(core::net::Ipv4Addr::new(172, 16, 0, 1), 17, 2);
        tbl5.insert(core::net::Ipv4Addr::new(192, 168, 1, 1), 17, 3);
        tbl5.insert(core::net::Ipv4Addr::new(8, 8, 8, 8), 17, 4);
        tbl5.insert(core::net::Ipv4Addr::new(1, 1, 1, 1), 17, 4);
        assert_ne!(tbl1, tbl5);


        // IPV6
        let mut tbl6 = IpLookupTable::<core::net::Ipv6Addr, i32>::new();
        tbl6.insert(core::net::Ipv6Addr::new(0x2001, 0x0db8, 0x85a3, 0, 0, 0x8a2e, 0x0370, 0x7334), 128, 1);
        tbl6.insert(core::net::Ipv6Addr::new(0x2607, 0xf8b0, 0x400d, 0x0, 0x0, 0x0, 0x0, 0x200e), 128, 2);
        tbl6.insert(core::net::Ipv6Addr::new(0x2a00, 0x1450, 0x4001, 0x80b, 0x0, 0x0, 0x0, 0x2003), 128, 3);
        tbl6.insert(core::net::Ipv6Addr::new(0x2404, 0x6800, 0x4003, 0x802, 0x0, 0x0, 0x0, 0x200e), 128, 4);

        let mut tbl7 = IpLookupTable::<core::net::Ipv6Addr, i32>::new();
        tbl7.insert(core::net::Ipv6Addr::new(0x2607, 0xf8b0, 0x400d, 0x0, 0x0, 0x0, 0x0, 0x200e), 128, 2);
        tbl7.insert(core::net::Ipv6Addr::new(0x2001, 0x0db8, 0x85a3, 0, 0, 0x8a2e, 0x0370, 0x7334), 128, 1);
        tbl7.insert(core::net::Ipv6Addr::new(0x2404, 0x6800, 0x4003, 0x802, 0x0, 0x0, 0x0, 0x200e), 128, 4);
        tbl7.insert(core::net::Ipv6Addr::new(0x2a00, 0x1450, 0x4001, 0x80b, 0x0, 0x0, 0x0, 0x2003), 128, 3);
        assert_eq!(tbl6, tbl7);

        let mut tbl8 = IpLookupTable::<core::net::Ipv6Addr, i32>::new();
        tbl8.insert(core::net::Ipv6Addr::new(0x2001, 0x0db8, 0x85a3, 0, 0, 0x8a2e, 0x0370, 0x7334), 128, 1);
        tbl8.insert(core::net::Ipv6Addr::new(0x2607, 0xf8b0, 0x400d, 0x0, 0x0, 0x0, 0x0, 0x200e), 128, 200);
        tbl8.insert(core::net::Ipv6Addr::new(0x2a00, 0x1450, 0x4001, 0x80b, 0x0, 0x0, 0x0, 0x2003), 128, 3);
        tbl8.insert(core::net::Ipv6Addr::new(0x2404, 0x6800, 0x4003, 0x802, 0x0, 0x0, 0x0, 0x200e), 128, 4);
        assert_ne!(tbl6, tbl8);

    }
}
