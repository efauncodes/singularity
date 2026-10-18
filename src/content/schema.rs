use bevy::prelude::Resource;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Resource)]
#[serde(deny_unknown_fields)]
pub struct GameConfig {
    pub starting_credits: u32,
    pub cargo_capacity: u32,
    pub starting_station_id: String,
    pub real_millis_per_unit: u32,
    pub goods: Vec<GoodConfig>,
    pub stations: Vec<StationConfig>,
    pub routes: Vec<RouteConfig>,
}

impl GameConfig {
    pub fn travel_units(&self, from: &str, to: &str) -> Option<u32> {
        self.routes.iter().find_map(|route| {
            let connects = (route.from_station_id == from && route.to_station_id == to)
                || (route.from_station_id == to && route.to_station_id == from);
            connects.then_some(route.travel_units)
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteConfig {
    pub from_station_id: String,
    pub to_station_id: String,
    pub travel_units: u32,
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
