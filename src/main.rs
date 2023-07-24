use std::array::from_fn;
use std::mem::swap;
use std::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};
mod prng;
use prng::Prng;

// gather: p0 + p015 + 4*p23 + p5 (lat 20)
// load aligned: 1*p015+1*p23 (lat ptr: 5, index: 8)
use int_lib::Int;
mod int_lib {
    use std::arch::x86_64::*;
    use std::fmt::Debug;
    use std::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};

    #[derive(Copy, Clone, Debug)]
    struct M256w(__m256i);
    impl Eq for M256w {
        fn assert_receiver_is_total_eq(&self) {
            todo!()
        }
    }
    impl PartialEq for M256w {
        fn eq(&self, other: &Self) -> bool {
            todo!()
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
        fn sll1(self) -> Self {
            Self(unsafe { _mm256_slli_si256::<1>(self.0) })
        }

        /// 1 lat, 1 cycle
        /// p5
        #[inline(always)]
        fn srl1(self) -> Self {
            Self(unsafe { _mm256_srli_si256::<1>(self.0) })
        }

        /// 1 lat, 1/3 cycle
        /// p015
        #[inline(always)]
        fn andn(self, other: Self) -> Self {
            Self(unsafe { _mm256_andnot_si256(other.0, self.0) })
        }
    }

    // lat: 3 cycles
    #[inline(always)]
    fn full_add<T: Int>(a: T, b: T, c: T) -> (T, T) {
        let a_xor_b = a ^ b;
        let t0 = a & b;
        // [dependency barrier]
        let t1 = a_xor_b ^ c;
        let t2 = a_xor_b & c;
        // [dependency barrier]
        (t1, t2 | t0)
    }
    #[inline(always)]
    fn full_add_inner<T: Int>(s: T) -> (T, T) {
        full_add(s.sll1(), s, s.srl1())
    }

    #[inline(never)]
    fn next_state_from_partials<T: Int>(s: T, p1: (T, T), p2: (T, T), p3: (T, T)) -> T {
        //// TODO: share full sum calc between row pairs (saves 1 operation/row on average)
        let (a, b) = full_add(p1.0, p2.0, p3.0); // 1, 2
        let (c, d) = full_add(p1.1, p2.1, p3.1); // 2, 4
        let bxc = b ^ c;
        a & bxc & !d | !a & !bxc & ((b & c) ^ d) & s
    }
    pub fn next_state<T: Int>(n1: T, n2: T, n3: T) -> T {
        next_state_from_partials(
            n2,
            full_add_inner(n1),
            full_add_inner(n2),
            full_add_inner(n3),
        )
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
        const BITS: usize = std::mem::size_of::<Self>();
        /// 1 lat, 1/3 cycle
        #[inline(always)]
        fn andn(self, other: Self) -> Self {
            self & (!other)
        }
        fn sll1(self) -> Self;
        fn srl1(self) -> Self;
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
        fn sll1(self) -> Self {
            self << T::from(1)
        }

        fn srl1(self) -> Self {
            self >> T::from(1)
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
}



#[cfg(test)]
fn nxt_state_line_nested(n1: u64, n2: u64, n3: u64) -> u64 {
    let s1 = sfadd(n1); // 1, 2
    let s2 = sfadd(n2); // 1, 2
    let s3 = sfadd(n3); // 1, 2

    calc_inner(s1, s2, s3, n2)
}

fn calc_inner(s1: (u64, u64), s2: (u64, u64), s3: (u64, u64), n2: u64) -> u64 {
    // 1, 2
    let (r1, s4_1) = fadd(s1.0, s2.0, s3.0);

    // 2, 4
    let s5 = hadd(s1.1, s2.1);
    // 2, 4
    let s6 = hadd(s3.1, s4_1);

    // 2, 4
    let (r2, s7_1) = hadd(s5.0, s6.0);

    let r3 = s5.1 ^ s6.1 ^ s7_1;
    //let r4 = (s5.1 & s6.1) ^ ((s5.1 ^ s6.1) & s7_1);
    let r4 = !((s5.1 & s6.1) ^ ((s5.1 ^ s6.1) & s7_1));

    (r1 & r2 & !r3 | !r1 & !r2 & r3 & n2) & r4 // 4 | (3 & alive)
}
#[cfg(test)]
fn nxt_state_line_optim(n1: u64, n2: u64, n3: u64) -> u64 {
    // alive(2, 3) -> alive
    // dead(3) -> alive
    //
    // ->
    // alive(3, 4) -> alive
    // dead(3) -> alive

    let s1 = sfadd(n1); // 1, 2
    let s2 = sfadd(n2); // 1, 2
    let s3 = sfadd(n3); // 1, 2

    let (r1, s4_1) = fadd(s1.0, s2.0, s3.0); // 1, 2

    let s5 = hadd(s1.1, s2.1); // 2, 4
    let s6 = hadd(s3.1, s4_1); // 2, 4

    let (r2, s7_1) = hadd(s5.0, s6.0); // 2, 4

    let r3 = s5.1 ^ s6.1 ^ s7_1;
    //let r4 = (s5.1 & s6.1) ^ ((s5.1 ^ s6.1) & s7_1);
    let r4 = !((s5.1 & s6.1) ^ ((s5.1 ^ s6.1) & s7_1));

    (r1 & r2 & !r3 | !r1 & !r2 & r3 & n2) & r4
}

#[inline(never)]
fn nxt_state_line(n1: u64, n2: u64, n3: u64) -> u64 {
    // alive(2, 3) -> alive
    // dead(3) -> alive
    //
    // ->
    // alive(3, 4) -> alive
    // dead(3) -> alive

    let s1 = sfadd(n1); // 1, 2
    let s2 = sfadd(n2); // 1, 2
    let s3 = sfadd(n3); // 1, 2

    let s4 = fadd(s1.0, s2.0, s3.0); // 1, 2

    let s5 = hadd(s1.1, s2.1); // 2, 4
    let s6 = hadd(s3.1, s4.1); // 2, 4

    let s7 = hadd(s5.0, s6.0); // 2, 4

    let s8 = fadd(s5.1, s6.1, s7.1); // 4, 8

    let r1 = s4.0;
    let r2 = s7.0;
    let r3 = s8.0;
    let r4 = s8.1;

    let eq3 = r1 & r2 & (!r3) & (!r4);
    let eq4 = (!r1) & (!r2) & r3 & (!r4);

    eq3 | (eq4 & n2)
}
fn hadd<T>(a: T, b: T) -> (T, T)
where
    T: Int,
{
    (a ^ b, a & b)
}
fn fadd<T>(a: T, b: T, c: T) -> (T, T)
where
    T: Int,
{
    (a ^ b ^ c, (a & b) ^ ((a ^ b) & c))
}
fn sfadd<T>(a: T) -> (T, T)
where
    T: Int,
{
    fadd(a.sll1(), a, a.srl1())
}

fn update_state(state: &[u64; 64], new_state: &mut [u64; 64]) {
    for (i, new_state) in new_state.iter_mut().enumerate() {
        *new_state = nxt_state_line(
            state[(state.len() - 1 + i) % state.len()],
            state[(state.len() + 0 + i) % state.len()],
            state[(state.len() + 1 + i) % state.len()],
        );
    }
}

fn main() {
    #[inline(never)]
    fn inner(a: u64, b: u64, c: u64) {
        println!("...\n");
    }
    inner(23489, 234234, 234234);
}

fn display_bits<const LEN: usize>(arr: &[u64; LEN]) {
    use std::fmt::Write;
    let mut buffer = String::new();
    let _ = writeln!(&mut buffer);
    let _ = writeln!(&mut buffer);
    for i in 0..LEN / 2 {
        for j in 0..64 {
            let _ = write!(
                &mut buffer,
                "{}",
                match (arr[i * 2] >> j & 1 != 0, arr[i * 2 + 1] >> j & 1 != 0) {
                    (false, false) => ' ',
                    (true, false) => '▀',
                    (false, true) => '▄',
                    (true, true) => '█',
                }
            );
        }
        let _ = writeln!(&mut buffer);
    }
    println!("{}", buffer);
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_bitwise() {
        let mut rng = Prng(23849);

        for _ in 0..1000 {
            let n1 = rng.next();
            let n2 = rng.next();
            let n3 = rng.next();
            let baseline = nxt_state_line_baseline(n1, n2, n3);
            assert_eq!(baseline, nxt_state_line(n1, n2, n3));
        }
    }
    #[test]
    fn test_bitwise_optim() {
        let mut rng = Prng(23849489348);

        for _ in 0..1000 {
            let n1 = rng.next();
            let n2 = rng.next();
            let n3 = rng.next();
            let baseline = nxt_state_line_baseline(n1, n2, n3);
            assert_eq!(baseline, nxt_state_line_optim(n1, n2, n3));
            assert_eq!(baseline, nxt_state_line_nested(n1, n2, n3));
            assert_eq!(baseline, int_lib::next_state(n1, n2, n3));
        }
    }
    fn nxt_state_line_baseline(n1: u64, n2: u64, n3: u64) -> u64 {
        let n1: [bool; 64] = from_fn(|i| n1 & (1 << i) != 0);
        let n2: [bool; 64] = from_fn(|i| n2 & (1 << i) != 0);
        let n3: [bool; 64] = from_fn(|i| n3 & (1 << i) != 0);

        let combined = [n1, n2, n3];

        let mut n_next: [bool; 64] = from_fn(|_| false);

        for (i, state) in n2.into_iter().enumerate() {
            let mut sum = 0;
            for (dy, dx) in [
                (2, -1),
                (2, 0),
                (2, 1),
                (1, -1),
                (1, 1),
                (0, -1),
                (0, 0),
                (0, 1),
            ] {
                sum += combined[dy]
                    .get((dx + i as i32) as usize)
                    .copied()
                    .unwrap_or_default() as usize;
            }

            n_next[i] = (state && (sum == 2 || sum == 3)) || (!state && sum == 3);
        }

        n_next
            .into_iter()
            .enumerate()
            .fold(0, |acc, (i, b)| acc | ((b as u64) << i))
    }
}


// p5: shift/permute
// p015: bitwise ops
// p23: load packed
// p237+p4: store
//
// p0 p1 p2 p3 p4 p5 p6 p7 | TP | lat  | instr   | desc
//                A        | 1  | 1    | VPSLLDQ | shift, permute
// A  A           A        | 3  | 1    | VPAND   | bitwise ops
//       A  A              | 2  | 5/8  | VMOVDQA | load packed
//       A  A  B        A  | 1  | 4/10 | VMOVDQA | store packed
// A  A                    |    |      |         | compare     
//
// p0 int multiply
// p2 load
// p3 load
// p5 shuffle unit
// p6 predicted taken
