mod prng;
mod ui;
mod int_lib;

use prng::Prng;
use ui::display_bits;
pub use int_lib::Int;

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
#[inline(always)]
fn cint_full_add_inner<T: Int>(s: T) -> (T, T) {
    cint_full_add(s.sll1(), s, s.srl1())
}

#[inline(never)]
fn cint_next_state_from_partials<T: Int>(s: T, p1: (T, T), p2: (T, T), p3: (T, T)) -> T {
    //// TODO: share full sum calc between row pairs (saves 1 operation/row on average)
    let (a, b) = cint_full_add(p1.0, p2.0, p3.0); // 1, 2
    let (c, d) = cint_full_add(p1.1, p2.1, p3.1); // 2, 4
    let bxc = b ^ c;
    a & bxc & !d | !a & !bxc & ((b & c) ^ d) & s
}
pub fn cint_next_state<T: Int>(n1: T, n2: T, n3: T) -> T {
    cint_next_state_from_partials(
        n2,
        cint_full_add_inner(n1),
        cint_full_add_inner(n2),
        cint_full_add_inner(n3),
    )
}
fn cint_next_state1<T: Int, F: Fn(T, (T, T), (T, T), (T, T)) -> T>(n1: T, n2: T, n3: T, f: F) -> T {
    f(
        n2,
        cint_full_add_inner(n1),
        cint_full_add_inner(n2),
        cint_full_add_inner(n3),
    )
}

fn next_state_line_foo(n1: u64, n2: u64, n3: u64) -> u64 {
    // alive(2, 3) -> alive
    // dead(3) -> alive
    //
    // ->
    // alive(3, 4) -> alive
    // dead(3) -> alive

    let s1 = self_full_add_reference(n1); // 1, 2
    let s2 = self_full_add_reference(n2); // 1, 2
    let s3 = self_full_add_reference(n3); // 1, 2

    let s4 = full_add_reference(s1.0, s2.0, s3.0); // 1, 2

    let s5 = half_add_reference(s1.1, s2.1); // 2, 4
    let s6 = half_add_reference(s3.1, s4.1); // 2, 4

    let s7 = half_add_reference(s5.0, s6.0); // 2, 4

    let s8 = full_add_reference(s5.1, s6.1, s7.1); // 4, 8

    let r1 = s4.0;
    let r2 = s7.0;
    let r3 = s8.0;
    let r4 = s8.1;

    let eq3 = r1 & r2 & (!r3) & (!r4);
    let eq4 = (!r1) & (!r2) & r3 & (!r4);

    eq3 | (eq4 & n2)
}

fn main() {
    let mut rng = Prng(314159265);
    let mut board = [(); 64].map(|_| rng.next());
    let mut next_board = [0.0; 64].map(|_| rng.next());
    loop {
        for i in 0..64_usize {
            let n1 = board.get(i.wrapping_sub(1)).copied().unwrap_or(0);
            let n2 = board.get(i).copied().unwrap_or(0);
            let n3 = board.get(i + 1).copied().unwrap_or(0);

            next_board[i] = next_state_line_foo(n1, n2, n3);
        }
        board = next_board;

        display_bits(&board);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}


pub use reference::*;
mod reference {
    use super::*;
    pub fn half_add_reference<T>(a: T, b: T) -> (T, T)
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
    pub fn self_full_add_reference<T>(a: T) -> (T, T)
    where
        T: Int,
    {
        full_add_reference(a.sll1(), a, a.srl1())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::array::from_fn;

    #[test]
    fn test_bitwise() {
        let mut rng = Prng(23849);

        for _ in 0..1000 {
            let n1 = rng.next();
            let n2 = rng.next();
            let n3 = rng.next();
            let baseline = next_state_row_correct(n1, n2, n3);
            assert_eq!(baseline, next_state_line_foo(n1, n2, n3));
        }
    }
    #[test]
    fn test_bitwise_optim() {
        let mut rng = Prng(23849489348);

        for _ in 0..1000 {
            let n1 = rng.next();
            let n2 = rng.next();
            let n3 = rng.next();
            let baseline = next_state_row_correct(n1, n2, n3);
            assert_eq!(baseline, nxt_state_line_optimized(n1, n2, n3));
            assert_eq!(baseline, next_state_row_reference(n1, n2, n3));
            assert_eq!(baseline, cint_next_state(n1, n2, n3));
        }
    }

    fn next_state_row_correct(n1: u64, n2: u64, n3: u64) -> u64 {
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

    fn next_state_row_reference(n1: u64, n2: u64, n3: u64) -> u64 {
        let s1 = self_full_add_reference(n1); // 1, 2
        let s2 = self_full_add_reference(n2); // 1, 2
        let s3 = self_full_add_reference(n3); // 1, 2

        next_state_from_partials_reference(s1, s2, s3, n2)
    }

    fn next_state_from_partials_reference(
        s1: (u64, u64),
        s2: (u64, u64),
        s3: (u64, u64),
        n2: u64,
    ) -> u64 {
        // 1, 2
        let (r1, s4_1) = full_add_reference(s1.0, s2.0, s3.0);

        // 2, 4
        let s5 = half_add_reference(s1.1, s2.1);
        // 2, 4
        let s6 = half_add_reference(s3.1, s4_1);

        // 2, 4
        let (r2, s7_1) = half_add_reference(s5.0, s6.0);

        let r3 = s5.1 ^ s6.1 ^ s7_1;
        //let r4 = (s5.1 & s6.1) ^ ((s5.1 ^ s6.1) & s7_1);
        let r4 = !((s5.1 & s6.1) ^ ((s5.1 ^ s6.1) & s7_1));

        (r1 & r2 & !r3 | !r1 & !r2 & r3 & n2) & r4 // 4 | (3 & alive)
    }

    fn nxt_state_line_optimized(n1: u64, n2: u64, n3: u64) -> u64 {
        // alive(2, 3) -> alive
        // dead(3) -> alive
        //
        // ->
        // alive(3, 4) -> alive
        // dead(3) -> alive

        let s1 = self_full_add_reference(n1); // 1, 2
        let s2 = self_full_add_reference(n2); // 1, 2
        let s3 = self_full_add_reference(n3); // 1, 2

        let (r1, s4_1) = full_add_reference(s1.0, s2.0, s3.0); // 1, 2

        let s5 = half_add_reference(s1.1, s2.1); // 2, 4
        let s6 = half_add_reference(s3.1, s4_1); // 2, 4

        let (r2, s7_1) = half_add_reference(s5.0, s6.0); // 2, 4

        let r3 = s5.1 ^ s6.1 ^ s7_1;
        //let r4 = (s5.1 & s6.1) ^ ((s5.1 ^ s6.1) & s7_1);
        let r4 = !((s5.1 & s6.1) ^ ((s5.1 ^ s6.1) & s7_1));

        (r1 & r2 & !r3 | !r1 & !r2 & r3 & n2) & r4
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
//
// gather: p0 + p015 + 4*p23 + p5 (lat 20)
// load aligned: 1*p015+1*p23 (lat ptr: 5, index: 8)
