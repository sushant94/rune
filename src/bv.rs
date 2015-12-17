//! Module that defines BV (BitVector) and the operations needed to operate on
//! them.

use petgraph::graph::Graph;
use std::convert::Into;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Shl, Shr, Sub};
use std::{u8, u16, u32, u64};
use std::ops::Range;

/// BitVectorValue (BVV) represents a concrete value
#[derive(Copy, Clone, Debug, Hash, PartialEq)]
pub struct BVV {
    value: u64,
    size: usize,
}

#[macro_export]
macro_rules! bvv_new {
    (u8 $t: expr) => ( BVV::new($t as u8, Some(8)) );
    (u16 $t: expr) => ( BVV::new($t as u16, Some(16)) );
    (u32 $t: expr) => ( BVV::new($t as u32, Some(32)) );
    (u64 $t: expr) => ( BVV::new($t as u64, Some(64)) );
}


impl BVV {
    fn new<T: Into<u64> + Copy>(val: T, size: Option<usize>) -> BVV {
        let size = size.unwrap_or(64);
        let max = match size {
            8 =>  u8::MAX as u64,
            16 => u16::MAX as u64,
            32 => u32::MAX as u64,
            64 => u64::MAX as u64,
            _ => panic!("BVV Error: Invalid size"),
        };
        assert!(val.into() <= max, "BVV Error: Value too large for given size");
        BVV { value: val.into(), size: size }
    }
}


/// BitVectorSymbol (BVS) represents a symbolic value
#[derive(Clone, Debug)]
pub struct BVS {
    /// Optional name for the symbolic var
    name: Option<String>,
    /// Index that represents the root for this symbolic var
    ast: Graph<usize, usize>,
}

macro_rules! bvv_binop {
    ($( impl $t: ident, $f: ident as $g: ident)*) => ($(
            impl $t<BVV> for BVV {
                type Output = BVV;
                fn $f(self, other: BVV) -> BVV {
                    assert!(self.size == other.size, "BVV Error: Incompatible sizes");
                    let result = match self.size {
                        8  =>  ((self.value as u8).$g(other.value as u8) as u64),
                        16 => ((self.value as u16).$g(other.value as u16) as u64),
                        32 => ((self.value as u32).$g(other.value as u32) as u64),
                        64 => ((self.value as u64).$g(other.value as u64) as u64),
                        _  => panic!("BVV Error: Invalid size for BVV"),
                    };
                    BVV::new(result, Some(self.size))
                }
            }
            )*)
}

impl Shl<BVV> for BVV {
    type Output = BVV;
    fn shl(self, other: BVV) -> BVV {
        assert!(self.size == other.size, "BVV Error: Incompatible sizes");
        let result = match self.size {
            8  =>  ((self.value as u8).wrapping_shl(other.value as u32) as u64),
            16 => ((self.value as u16).wrapping_shl(other.value as u32) as u64),
            32 => ((self.value as u32).wrapping_shl(other.value as u32) as u64),
            64 => ((self.value as u64).wrapping_shl(other.value as u32) as u64),
            _  => panic!("BVV Error: Invalid size for BVV"),
        };
        BVV::new(result, Some(self.size))
    }
}

impl Shr<BVV> for BVV {
    type Output = BVV;
    fn shr(self, other: BVV) -> BVV {
        assert!(self.size == other.size, "BVV Error: Incompatible sizes");
        let result = match self.size {
            8  =>  ((self.value as u8).wrapping_shr(other.value as u32) as u64),
            16 => ((self.value as u16).wrapping_shr(other.value as u32) as u64),
            32 => ((self.value as u32).wrapping_shr(other.value as u32) as u64),
            64 => ((self.value as u64).wrapping_shr(other.value as u32) as u64),
            _  => panic!("BVV Error: Invalid size for BVV"),
        };
        BVV::new(result, Some(self.size))
    }
}

bvv_binop!{ impl Add, add as wrapping_add
            impl BitAnd, bitand as bitand
            impl BitOr, bitor as bitor
            impl BitXor, bitxor as bitxor
            impl Div, div as wrapping_div
            impl Mul, mul as wrapping_mul
            impl Rem, rem as wrapping_rem
            impl Sub, sub as wrapping_sub
}

impl Not for BVV {
    type Output = Self;
    #[inline]
    fn not(self) -> BVV { 
        let mut result = !self.value;
        let m = (1 << (self.size - 1)) - 1;
        result = result & m;
        BVV::new(result, Some(self.size))
    }
}

impl BVV {
    pub fn at(&self, idx: usize) -> u8 {
        assert!(idx < self.size as usize, "BVV Error: Index Out-of-Bounds");
        (((self.value >> idx) & 1) as u8)
    }

    pub fn set(&mut self, idx: usize, v: u8) {
        assert!(idx < self.size as usize, "BVV Error: Index Out-of-Bounds");
        assert!(v < 2);
        if self.at(idx) != v {
            // Flip the bit
            let mask = 1 << idx;
            self.value = self.value ^ mask;
        }
    }

    // Returns a BitVector with the smallest valid size
    pub fn range(&self, r: Range<usize>) -> BVV {
        // Range is Half-open [_, _)
        assert!(r.end <= self.size as usize, "BVV Error: Range Index Out-of-Bounds");
        let start = r.start;
        let end = r.end;
        let result = r.fold(0, |acc, v| acc | (self.at(v) << (v - start) as u64));
        let mut new_size = (end - start).next_power_of_two();
        if new_size < 8 {
            new_size = 8;
        }
        BVV::new(result, Some(new_size))
    }

    pub fn set_range(&mut self, r: Range<usize>, v: u64) {
        assert!(r.end <= self.size as usize, "BVV Error: Range Index Out-of-Bounds");
        let start = r.start;
        for i in r {
            self.set(i, ((v >> (i - start)) & 1) as u8);
        }
    }

    pub fn sign_extend(&mut self, new_size: usize) {
        assert!(new_size > self.size as usize, "BVV Error: Cannot extend to a narrower size");
        let valid_sizes = vec![8, 16, 32, 64];
        assert!(valid_sizes.contains(&new_size), "BVV Error: Invalid size");
        let old_size = self.size as usize;
        let sign = self.at(old_size - 1);
        self.size = new_size;
        if sign == 1 {
            self.set_range((old_size..new_size), u64::MAX);
        }
    }
    
    pub fn zero_extend(&mut self, new_size: usize) {
        assert!(new_size > self.size as usize, "BVV Error: Cannot extend to a narrower size");
        let valid_sizes = vec![8, 16, 32, 64];
        assert!(valid_sizes.contains(&new_size), "BVV Error: Invalid size");
        self.size = new_size;
    }
}

///////////////////////////////////////////////////////////////////////////////
//// Trait implementations for BVS
///////////////////////////////////////////////////////////////////////////////


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bvv_add() {
        let x = bvv_new!(u16 5);
        let y = bvv_new!(u16 6);
        assert_eq!(x + y, bvv_new!(u16 11));
    }

    #[test]
    fn bvv_add_overflow() {
        let x = bvv_new!(u8 250);
        let y = bvv_new!(u8 10);
        assert_eq!(x + y, bvv_new!(u8 4));
    }

    #[test]
    #[should_panic]
    fn bvv_invalid_size() {
        BVV::new(256u64, Some(8));
    }

    #[test]
    fn bvv_not() {
        let x = bvv_new!(u8 1);
        assert_eq!(!x, bvv_new!(u8 126));
    }

    #[test]
    fn bvv_set() {
        let mut x = bvv_new!(u8 1);
        x.set(1, 1);
        assert_eq!(x, bvv_new!(u8 3));
        x.set(1, 1);
        assert_eq!(x, bvv_new!(u8 3));
        x.set(0, 0);
        assert_eq!(x, bvv_new!(u8 2));
        x.set(1, 0);
        assert_eq!(x, bvv_new!(u8 0));
        x.set(1, 0);
        assert_eq!(x, bvv_new!(u8 0));
    }

    #[test]
    fn bvv_at() {
        let x = bvv_new!(u8 5);
        assert_eq!(x.at(0), 1);
        assert_eq!(x.at(2), 1);
        assert_eq!(x.at(1), 0);
    }

    #[test]
    fn bvv_range() {
        let x = bvv_new!(u8 0b01001011);
        assert_eq!(x.range((0..4)), bvv_new!(u8 0b00001011));
        assert_eq!(x.range((4..8)), bvv_new!(u8 0b00000100));
    }
    
    #[test]
    fn bvv_set_range() {
        let mut x = bvv_new!(u8 0b00001011);
        x.set_range((4..8), 0b0100);
        assert_eq!(x, bvv_new!(u8 0b01001011));
        x.set_range((0..4), 0b0001);
        assert_eq!(x, bvv_new!(u8 0b01000001));
    }

    #[test]
    fn bvv_sign_extend() {
        // x = -4
        let mut x = bvv_new!(u8 !0b00000011);
        x.sign_extend(16);
        assert_eq!(x.value as i16, -4);
        // x = 3
        let mut x = bvv_new!(u8 0b00000011);
        x.sign_extend(16);
        assert_eq!(x.value as i16, 3);
    }

    #[test]
    fn bvv_zero_extend() {
        // x = -4
        let mut x = bvv_new!(u8 0b11111100);
        x.zero_extend(16);
        assert_eq!(x.value as i16, 0b000000011111100);
        // x = 3
        let mut x = bvv_new!(u8 0b00000011);
        x.zero_extend(16);
        assert_eq!(x.value as i16, 0b00000011);
    }
}
