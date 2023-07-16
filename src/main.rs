use std::array::from_fn;
use std::mem::swap;

struct Prng(pub u64);

impl Prng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

#[test]
fn gol_tests() {
    let mut rng = Prng(23);

    for i in 0..10 {
        let n1 = rng.next();
        let n2 = rng.next();
        let n3 = rng.next();
        assert_eq!(
            nxt_state_line(n1, n2, n3),
            nxt_state_line_baseline(n1, n2, n3)
        );
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
fn hadd(a: u64, b: u64) -> (u64, u64) {
    (a ^ b, a & b)
}
fn fadd(a: u64, b: u64, c: u64) -> (u64, u64) {
    (a ^ b ^ c, (a & b) ^ ((a ^ b) & c))
}
fn sfadd(a: u64) -> (u64, u64) {
    fadd(a << 1, a, a >> 1)
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
    println!("Hello, world!▄▀ █");
    let mut prng = Prng(2348928348923895);
    let mut state = [(); 64].map(|_| prng.next());
    let mut state1 = [0; 64];
    loop {
        display_bits(&state);
        for _ in 0..1 {
            state[0] = prng.next();
            update_state(&state, &mut state1);
            swap(&mut state, &mut state1);
        }
        std::thread::sleep(std::time::Duration::from_secs_f32(2.0 / 60.0));
    }
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
