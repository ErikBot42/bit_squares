use itertools::Itertools;
use std::iter::once;
// lat: 3 cycles
#[inline(always)]
fn cint_full_add<T: Int>(a: T, b: T, c: T) -> (T, T) {
    let a_xor_b = a ^ b;
    let t0 = a & b;
    // [dependency barrier]
    let t1 = a_xor_b ^ c;
    let t2 = a_xor_b & c;
    // [dependency barrier]
    (t1, t2 | t0)
}

fn cint_next_state_from_partials<T: Int>(s: T, p1: (T, T), p2: (T, T), p3: (T, T)) -> T {
    let (a, b) = cint_full_add(p1.0, p2.0, p3.0); // 1, 2
    let (c, d) = cint_full_add(p1.1, p2.1, p3.1); // 2, 4
    let bxc = b ^ c;
    a & bxc & !d | !a & !bxc & ((b & c) ^ d) & s
}
pub fn cint_next_state<T: Int>(n1: T, n2: T, n3: T) -> T {
    cint_next_state1(n1, n2, n3, cint_next_state_from_partials)
}
fn full_add_inner<T: Int>(s: T) -> (T, T) {
    cint_full_add(s.sll(), s, s.srl())
}
fn cint_next_state1<T: Int, F: Fn(T, (T, T), (T, T), (T, T)) -> T>(
    n1: T,
    n2: T,
    n3: T,
    f: F,
) -> T {
    f(
        n2,
        full_add_inner(n1),
        full_add_inner(n2),
        full_add_inner(n3),
    )
}

fn next_state_line_simple(n1: u64, n2: u64, n3: u64) -> u64 {
    let s1 = full_add_row_reference(n1); // 1, 2
    let s2 = full_add_row_reference(n2); // 1, 2
    let s3 = full_add_row_reference(n3); // 1, 2
    let s4 = full_add_reference(s1.0, s2.0, s3.0); // 1, 2

    let s5 = half_add(s1.1, s2.1); // 2, 4
    let s6 = half_add(s3.1, s4.1); // 2, 4
    let s7 = half_add(s5.0, s6.0); // 2, 4
    let s8 = full_add_reference(s5.1, s6.1, s7.1); // 4, 8

    let r1 = s4.0;
    let r2 = s7.0;
    let r3 = s8.0;
    let r4 = s8.1;

    let eq3 = r1 & r2 & (!r3) & (!r4);
    let eq4 = (!r1) & (!r2) & r3 & (!r4);

    eq3 | (eq4 & n2)
}

pub fn streaming_next_state<T: Int>(iter: impl Iterator<Item = T>) -> impl Iterator<Item = T> {
    let next_state = iter
        .map(|a| (a, full_add_inner(a)))
        .tuple_windows()
        .map(|((_, p1), (s, p2), (_, p3))| cint_next_state_from_partials(s, p1, p2, p3));

    next_state
}

fn update_board_simple(board: &mut [u64; 64], next_board: &mut [u64; 64]) {
    for i in 0..64_usize {
        let n1 = board.get(i.wrapping_sub(1)).copied().unwrap_or(0);
        let n2 = board.get(i).copied().unwrap_or(0);
        let n3 = board.get(i + 1).copied().unwrap_or(0);

        next_board[i] = next_state_line_simple(n1, n2, n3);
    }

    *board = *next_board;
}
use crate::Int;
// 64 * 64 = 4096 bits = 512 bytes = 8 cache lines
// 256 * 256 = 65536 bits = 8192 bytes = 128 cache lines
#[repr(C)]
#[derive(Clone, Eq, PartialEq)] // not deriving copy because of size
pub struct Tile<const H: usize, I: Int>([I; H]);
impl<const H: usize, I: Int> Tile<H, I> {
    fn new() -> Self {
        Self([I::zero(); H])
    }
    fn update_n<const N: usize>(&mut self) {}
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&row| row == I::zero())
    }
}

#[inline(never)]
pub fn update_board_stream(board: &mut [u64; 64], next_board: &mut [u64; 64]) {
    for (i, nxt_state) in
        streaming_next_state(once(0).chain(board.iter().copied()).chain(once(0))).enumerate()
    {
        next_board[i] = nxt_state;
    }
    *board = *next_board;
}

pub fn half_add<T>(a: T, b: T) -> (T, T)
where
    T: Int,
{
    (a ^ b, a & b)
}
pub fn full_add_reference<T>(a: T, b: T, c: T) -> (T, T)
where
    T: Int,
{
    (a ^ b ^ c, (a & b) ^ ((a ^ b) & c))
}
pub fn full_add_row_reference<T>(a: T) -> (T, T)
where
    T: Int,
{
    full_add_reference(a.sll(), a, a.srl())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Prng;
    use std::array::from_fn;

    #[test]
    fn test_bitwise() {
        let mut rng = Prng(23849);

        for _ in 0..1000 {
            let n1 = rng.next();
            let n2 = rng.next();
            let n3 = rng.next();
            let baseline = next_row_without_bitwise(n1, n2, n3);
            assert_eq!(baseline, next_state_line_simple(n1, n2, n3));
        }
    }
    #[test]
    fn test_bitwise_optim() {
        let mut rng = Prng(23849489348);

        for _ in 0..1000 {
            let n1 = rng.next();
            let n2 = rng.next();
            let n3 = rng.next();
            let baseline = next_row_without_bitwise(n1, n2, n3);
            assert_eq!(baseline, next_state_line_optimized(n1, n2, n3));
            assert_eq!(baseline, cint_next_state(n1, n2, n3));
            assert_eq!(baseline, next_state_line_simple(n1, n2, n3));
        }
    }
    #[test]
    fn test_full_board_update() {
        let seed = 314159265;

        for seed in seed..(seed + 10) {
            let mut rng = Prng(seed);
            let mut a_board = [(); 64].map(|_| rng.next());
            let mut a_next_board = [0.0; 64].map(|_| rng.next());
            let mut b_board = a_board;
            let mut b_next_board = a_next_board;
            for _ in 0..20 {
                update_board_simple(&mut a_board, &mut a_next_board);
                update_board_stream(&mut b_board, &mut b_next_board);
            }
            assert_eq!(a_board, b_board);
        }
    }

    fn next_row_without_bitwise(n1: u64, n2: u64, n3: u64) -> u64 {
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

    fn next_state_line_optimized(n1: u64, n2: u64, n3: u64) -> u64 {
        let s1 = full_add_row_reference(n1); // 1, 2
        let s2 = full_add_row_reference(n2); // 1, 2
        let s3 = full_add_row_reference(n3); // 1, 2

        let (r1, s4_1) = full_add_reference(s1.0, s2.0, s3.0); // 1, 2

        let s5 = half_add(s1.1, s2.1); // 2, 4
        let s6 = half_add(s3.1, s4_1); // 2, 4

        let (r2, s7_1) = half_add(s5.0, s6.0); // 2, 4

        let r3 = s5.1 ^ s6.1 ^ s7_1;
        //let r4 = (s5.1 & s6.1) ^ ((s5.1 ^ s6.1) & s7_1);
        let r4 = !((s5.1 & s6.1) ^ ((s5.1 ^ s6.1) & s7_1));

        (r1 & r2 & !r3 | !r1 & !r2 & r3 & n2) & r4
    }
}
