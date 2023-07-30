use std::arch::x86_64::*;
use std::fmt::Debug;
use std::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};

#[derive(Copy, Clone, Debug)]
struct M256w(__m256i);
impl Eq for M256w {
    fn assert_receiver_is_total_eq(&self) {}
}
impl PartialEq for M256w {
    fn eq(&self, other: &Self) -> bool {
        -1_i32 == unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi64(self.0, other.0)) }
    }
}

impl BitOr for M256w {
    type Output = Self;

    /// 1 lat, 1/3 cycle
    /// p015
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(unsafe { _mm256_or_si256(self.0, rhs.0) })
    }
}
impl BitXor for M256w {
    type Output = Self;

    /// 1 lat, 1/3 cycle
    /// p015
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(unsafe { _mm256_xor_si256(self.0, rhs.0) })
    }
}
impl BitAnd for M256w {
    type Output = Self;

    /// 1 lat, 1/3 cycle
    /// p015
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(unsafe { _mm256_and_si256(self.0, rhs.0) })
    }
}
impl Not for M256w {
    type Output = Self;

    /// 1 lat, 1/3 cycle
    /// p015 or p015 + p23
    #[inline(always)]
    fn not(self) -> Self::Output {
        Self(unsafe { _mm256_xor_si256(self.0, _mm256_set1_epi32(-1)) })
    }
}
impl Int for M256w {
    /// 1 lat, 1 cycle
    /// p5
    #[inline(always)]
    fn sll(self) -> Self {
        Self(unsafe { _mm256_slli_si256::<1>(self.0) })
    }

    /// 1 lat, 1 cycle
    /// p5
    #[inline(always)]
    fn srl(self) -> Self {
        Self(unsafe { _mm256_srli_si256::<1>(self.0) })
    }

    /// 1 lat, 1/3 cycle
    /// p015
    #[inline(always)]
    fn andn(self, other: Self) -> Self {
        Self(unsafe { _mm256_andnot_si256(other.0, self.0) })
    }

    fn zero() -> Self {
        Self(unsafe { _mm256_setzero_si256() })
    }
}

pub trait Int:
    BitXor<Output = Self>
    + BitAnd<Output = Self>
    + Not<Output = Self>
    + BitOr<Output = Self>
    + Copy
    + Clone
    + Debug
    + PartialEq
    + Eq
{
    const BITS: usize = std::mem::size_of::<Self>() * 8;
    /// 1 lat, 1/3 cycle
    #[inline(always)]
    fn andn(self, other: Self) -> Self {
        self & (!other)
    }
    fn sll(self) -> Self;
    fn srl(self) -> Self;
    fn zero() -> Self;
    fn board<const BITS: usize>() -> [Self; BITS] {
        [Self::zero(); BITS]
    }
}

impl<T> Int for T
where
    T: BitXor<Output = T>
        + BitAnd<Output = T>
        + Not<Output = T>
        + BitOr<Output = T>
        + Shr<Output = T>
        + Shl<Output = T>
        + From<u8>
        + Copy
        + Clone
        + Debug
        + PartialEq
        + Eq,
{
    fn sll(self) -> Self {
        self << T::from(1)
    }

    fn srl(self) -> Self {
        self >> T::from(1)
    }

    fn zero() -> Self {
        0.into()
    }
}

#[derive(Debug, Clone)]
struct BitLangVar(String);

impl BitOr for BitLangVar {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        BitLangVar(format!("(| {} {})", &self.0, &rhs.0))
    }
}
impl BitXor for BitLangVar {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        BitLangVar(format!("(^ {} {})", &self.0, &rhs.0))
    }
}
impl BitAnd for BitLangVar {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        BitLangVar(format!("(& {} {})", &self.0, &rhs.0))
    }
}
impl Shr for BitLangVar {
    type Output = Self;

    fn shr(self, rhs: BitLangVar) -> Self::Output {
        BitLangVar(format!("(> {} {})", &self.0, &rhs.0))
    }
}
impl Shl for BitLangVar {
    type Output = Self;

    fn shl(self, rhs: BitLangVar) -> Self::Output {
        BitLangVar(format!("(< {} {})", &self.0, &rhs.0))
    }
}
impl Not for BitLangVar {
    type Output = Self;

    fn not(self) -> Self::Output {
        BitLangVar(format!("(! {})", &self.0))
    }
}
impl From<u8> for BitLangVar {
    fn from(value: u8) -> Self {
        BitLangVar(format!("{value}"))
    }
}
