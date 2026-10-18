use std::collections::HashMap;

use bevy::prelude::*;

use crate::content::{GoodConfig, StationConfig};

#[derive(Component)]
pub struct Player;

#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct PlayerState {
    pub credits: u32,
    pub cargo_capacity: u32,
    pub cargo: HashMap<String, u32>,
}

impl PlayerState {
    pub fn new(credits: u32, cargo_capacity: u32) -> Self {
        Self {
            credits,
            cargo_capacity,
            cargo: HashMap::new(),
        }
    }

    pub fn cargo_weight(&self, goods: &[GoodConfig]) -> u32 {
        goods.iter().fold(0_u32, |weight, good| {
            weight.saturating_add(
                self.cargo
                    .get(&good.id)
                    .copied()
                    .unwrap_or_default()
                    .saturating_mul(good.weight),
            )
        })
    }
}

#[derive(Component, Clone, Debug)]
pub struct StationState {
    pub id: String,
    pub name: String,
    pub stock: HashMap<String, u32>,
    pub player_buy_prices: HashMap<String, u32>,
    pub player_sell_prices: HashMap<String, u32>,
}

impl StationState {
    pub fn from_config(config: &StationConfig) -> Self {
        let stock = config
            .market
            .iter()
            .map(|entry| (entry.good_id.clone(), entry.stock))
            .collect();
        let player_buy_prices = config
            .market
            .iter()
            .map(|entry| (entry.good_id.clone(), entry.player_buy_price))
            .collect();
        let player_sell_prices = config
            .market
            .iter()
            .map(|entry| (entry.good_id.clone(), entry.player_sell_price))
            .collect();

        Self {
            id: config.id.clone(),
            name: config.name.clone(),
            stock,
            player_buy_prices,
            player_sell_prices,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TradeDirection {
    Buy,
    Sell,
}

#[derive(Message, Clone, Debug)]
pub struct TradeRequest {
    pub station: Entity,
    pub good_id: String,
    pub quantity: u32,
    pub direction: TradeDirection,
}

#[derive(Resource, Debug, PartialEq, Eq)]
pub struct TradeResult(pub String);

impl Default for TradeResult {
    fn default() -> Self {
        Self("Enter a quantity to trade at your docked station.".to_string())
    }
}
