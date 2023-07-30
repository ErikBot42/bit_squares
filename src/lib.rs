pub mod int_lib;
pub use int_lib::Int;

pub mod prng;
pub use prng::Prng;

pub mod ui;
pub use ui::display_bits;

pub use tile::{streaming_next_state, Tile};
pub mod tile;
pub mod board;

// alive(2, 3) -> alive
// dead(3) -> alive
//
// ->
// alive(3, 4) -> alive
// dead(3) -> alive

// TODO: share full sum calc between row pairs (saves 1 operation/row on average)

// p5: shift/permute
// p015: bitwise ops
// p23: load packed
// p237+p4: store
//
// p0 p1 p2 p3 p4 p5 p6 p7 | TP | lat  | instr            | desc
//                A        | 1  | 1    | VPSLLDQ          | shift, permute
// A  A           A        | 3  | 1    | VPAND            | bitwise ops
//       A  A              | 2  | 5/8  | VMOVDQA          | load packed
//       A  A  B        A  | 1  | 4/10 | VMOVDQA          | store packed
// A  A                    |    |      |                  | compare
// A  A           A        | 3  | 1    | X/OR/AND reg/imm |
//                         |    |      |                  |
//                         |    |      |                  |
//
// p0 int multiply
// p2 load
// p3 load
// p5 shuffle unit
// p6 predicted taken
//
// gather: p0 + p015 + 4*p23 + p5 (lat 20)
// load aligned: 1*p015+1*p23 (lat ptr: 5, index: 8)
