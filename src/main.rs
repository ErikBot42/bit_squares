use bit_squares::*;

fn main() {
    fn update_board(board: &mut [u64; 64], next_board: &mut [u64; 64]) {
        //for i in 0..64_usize {
        //    let n1 = board.get(i.wrapping_sub(1)).copied().unwrap_or(0);
        //    let n2 = board.get(i).copied().unwrap_or(0);
        //    let n3 = board.get(i + 1).copied().unwrap_or(0);

        //    next_board[i] = next_state_line_simple(n1, n2, n3);
        //}

        for (i, nxt_state) in streaming_next_state(board.iter().copied()).enumerate() {
            next_board[i] = nxt_state;
        }
        *board = *next_board;
    }
    let seed = 314159265;
    let mut rng = Prng(seed);
    let mut board = [(); 64].map(|_| rng.next());
    let mut next_board = [0.0; 64].map(|_| rng.next());
    loop {
        update_board(&mut board, &mut next_board);
        display_bits(&board);
        std::thread::sleep(std::time::Duration::from_secs_f64(0.5));
    }
}

