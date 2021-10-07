/// Inventory x,y coordinate to index.
// TODO: we should never have to use this, remove
pub fn xy_to_i(x: u32, y: u32) -> u8 {
    assert!(x < crate::INV_WIDTH as u32, "x coord out of bound");
    assert!(y < crate::INV_WIDTH as u32, "y coord out of bound");
    (y * crate::INV_WIDTH as u32 + x) as u8
}

/// Inventory index to x,y coordinate.
// TODO: we should never have to use this, remove
pub fn i_to_xy(i: u8) -> (u32, u32) {
    assert!(i < crate::INV_SIZE as u8, "index out of bound");
    (
        i as u32 % crate::INV_WIDTH as u32,
        i as u32 / crate::INV_WIDTH as u32,
    )
}

/// Produces `1.0`.
pub const fn one() -> f64 {
    1.0
}
