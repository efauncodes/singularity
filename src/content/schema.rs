use bevy::prelude::Resource;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Resource)]
#[serde(deny_unknown_fields)]
pub struct GameConfig {
    pub starting_credits: u32,
    pub cargo_capacity: u32,
    pub goods: Vec<GoodConfig>,
    pub stations: Vec<StationConfig>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoodConfig {
    pub id: String,
    pub name: String,
    pub weight: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StationConfig {
    pub id: String,
    pub name: String,
    pub market: Vec<MarketEntry>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarketEntry {
    pub good_id: String,
    pub stock: u32,
    pub player_buy_price: u32,
    pub player_sell_price: u32,
}
