/// Inventory x,y coordinate to index.
pub fn xy_to_i(x: u32, y: u32) -> usize {
    assert!(x < crate::INV_WIDTH as u32, "x coord out of bound");
    assert!(y < crate::INV_WIDTH as u32, "y coord out of bound");
    (y * crate::INV_WIDTH as u32 + x) as usize
}

/// Inventory index to x,y coordinate.
pub fn i_to_xy(i: usize) -> (u32, u32) {
    assert!(i < crate::INV_SIZE as usize, "index out of bound");
    (
        i as u32 % crate::INV_WIDTH as u32,
        i as u32 / crate::INV_WIDTH as u32,
    )
}
