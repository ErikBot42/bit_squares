use super::{Int, Tile};
use std::collections::HashMap;
struct Board<const H: usize, I: Int> {
    tiles: Vec<Tile<H, I>>,
    tile_lut: Vec<(i32, i32)>,
    tile_lut_inv: HashMap<(i32, i32), usize>,
    update_list: Vec<usize>,
    next_update_list: Vec<usize>,
    in_update_list: Vec<bool>,
}
impl<const H: usize, I: Int> Board<H, I> {
    fn new() -> Self {
        Self {
            tiles: Vec::new(),
            tile_lut: Vec::new(),
            tile_lut_inv: HashMap::new(),
            update_list: Vec::new(),
            next_update_list: Vec::new(),
            in_update_list: Vec::new(),
        }
    }
    fn set_tile(&mut self, tile: &Tile<H, I>, coord: (i32, i32)) {
        match self.tile_lut_inv.get(&coord) {
            Some(existing_tile) => {
                let index = self.tile_lut_inv[&coord];
                self.tiles[index].clone_from(tile);
                if !self.in_update_list[index] {
                    self.update_list.push(index)
                }
            }
            None => {}
        }

        if !tile.is_zero() {
            // recurse
        }

        todo!()
    }
}

