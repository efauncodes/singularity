use bevy::prelude::*;

use crate::content::GameConfig;
use crate::trade::{Player, PlayerState, StationState, TradePlugin};
use crate::ui::{TradeUiPlugin, setup_trade_ui};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((TradePlugin, TradeUiPlugin))
            .add_systems(Startup, (spawn_market, setup_trade_ui).chain());
    }
}

fn spawn_market(mut commands: Commands, config: Res<GameConfig>) {
    commands.spawn((
        Player,
        PlayerState::new(config.starting_credits, config.cargo_capacity),
        Name::new("Player"),
    ));

    for station in &config.stations {
        commands.spawn((
            StationState::from_config(station),
            Name::new(station.name.clone()),
        ));
    }
}
