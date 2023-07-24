pub fn display_bits<const LEN: usize>(arr: &[u64; LEN]) {
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

