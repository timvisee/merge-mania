use serde::Deserialize;

/// Inventory item.
#[derive(Deserialize, Debug)]
enum Item {
    Product(Product),
    Factory(Factory),
}

/// Inventory product.
#[derive(Deserialize, Debug)]
pub struct Product {
    tier: u16,
    level: u16,
}

/// Inventory factory.
#[derive(Deserialize, Debug)]
pub struct Factory {
    tier: u16,
    level: u16,
}

/// An inventory.
#[derive(Default)]
pub struct Inventory {
    items: [Option<Item>; crate::INV_SIZE as usize],
}

#[derive(Deserialize, Debug)]
pub struct LibProducts {
    tiers: Vec<LibProductTier>,
}

#[derive(Deserialize, Debug)]
pub struct LibProductTier {
    id: u32,
    name: String,
    products: Vec<LibProduct>,
}

#[derive(Deserialize, Debug)]
pub struct LibProduct {
    name: String,
    cost: u32,
    sprite_path: String,
}

#[derive(Deserialize, Debug)]
pub struct LibFactories {
    tiers: Vec<LibFactoryTier>,
}

#[derive(Deserialize, Debug)]
pub struct LibFactoryTier {
    id: u32,
    name: String,
    levels: Vec<LibFactory>,
}

#[derive(Deserialize, Debug)]
pub struct LibFactory {
    name: String,
    // TODO: may cost multiple items
    cost_buy: u32,
    cost_sell: u32,
    time: u32,
    drops: Vec<LibFactoryDrop>,
    sprite_path: String,
}

#[derive(Deserialize, Debug)]
pub struct LibFactoryDrop {
    // TODO: change to Item ?
    item: String,
    chance: f32,
}
