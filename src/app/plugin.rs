use bevy::prelude::*;

use crate::content::GameConfig;
use crate::ship::{Ship, ShipLocation, ShipPlugin};
use crate::simulation::SimulationPlugin;
use crate::trade::{Player, PlayerState, StationState, TradePlugin};
use crate::ui::{TradeUiPlugin, setup_trade_ui};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((GameCorePlugin, TradeUiPlugin))
            .add_systems(Startup, setup_trade_ui.after(spawn_market));
    }
}

/// The production game systems and world setup, without rendering or UI dependencies.
pub struct GameCorePlugin;

impl Plugin for GameCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((SimulationPlugin, ShipPlugin, TradePlugin))
            .add_systems(Startup, spawn_market);
    }
}

fn spawn_market(mut commands: Commands, config: Res<GameConfig>) {
    let mut starting_station = None;
    for station in &config.stations {
        let entity = commands
            .spawn((
                StationState::from_config(station),
                Name::new(station.name.clone()),
            ))
            .id();
        if station.id == config.starting_station_id {
            starting_station = Some(entity);
        }
    }

    let station = starting_station.expect("validated content must contain the starting station");
    commands.spawn((
        Player,
        Ship,
        PlayerState::new(config.starting_credits, config.cargo_capacity),
        ShipLocation::Docked { station },
        Name::new("Player ship"),
    ));
}
