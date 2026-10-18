use bevy::prelude::*;

use super::model::{Player, PlayerState, StationState, TradeRequest, TradeResult};
use super::rules::execute_trade;
use crate::content::GameConfig;
use crate::ship::ShipLocation;
use crate::simulation::SimulationSet;

pub struct TradePlugin;

impl Plugin for TradePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TradeRequest>()
            .init_resource::<TradeResult>()
            .add_systems(Update, process_trade_requests.in_set(SimulationSet::Trade));
    }
}

pub fn process_trade_requests(
    mut requests: MessageReader<TradeRequest>,
    config: Res<GameConfig>,
    mut player_query: Query<(&mut PlayerState, &ShipLocation), With<Player>>,
    mut station_query: Query<&mut StationState>,
    mut result: ResMut<TradeResult>,
) {
    let Ok((mut player, location)) = player_query.single_mut() else {
        return;
    };

    for request in requests.read() {
        if !location.is_docked_at(request.station) {
            result.0 = "Dock at that station before trading.".to_string();
            continue;
        }
        let Ok(mut station) = station_query.get_mut(request.station) else {
            result.0 = "That station is unavailable.".to_string();
            continue;
        };

        match execute_trade(&mut player, &mut station, &config.goods, request) {
            Ok(()) => {
                result.0 = format!(
                    "Trade complete: {} x {}.",
                    request.quantity, request.good_id
                )
            }
            Err(error) => result.0 = error.to_string(),
        }
    }
}
