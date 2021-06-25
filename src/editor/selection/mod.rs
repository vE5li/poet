mod mode;

use std::cmp::{ min, max };

pub use self::mode::SelectionMode;

#[derive(Clone, Debug)]
pub struct Selection {
    pub primary_index: usize,
    pub secondary_index: usize,
    pub offset: usize,
}

impl Selection {

    pub fn new(primary_index: usize, secondary_index: usize, offset: usize) -> Self {
        Self {
            primary_index: primary_index,
            secondary_index: secondary_index,
            offset: offset,
        }
    }
}
