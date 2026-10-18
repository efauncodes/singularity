use bevy::prelude::*;

use super::{SimulationClock, SimulationSet, TimeControl};
use crate::content::GameConfig;
use crate::ship::ShipLocation;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationClock>()
            .add_message::<TimeControl>()
            .configure_sets(
                Update,
                (
                    SimulationSet::Controls,
                    SimulationSet::Trade,
                    SimulationSet::TravelOrders,
                    SimulationSet::Advance,
                    SimulationSet::Arrivals,
                    SimulationSet::Notifications,
                    SimulationSet::Presentation,
                )
                    .chain(),
            )
            .add_systems(Update, control_time.in_set(SimulationSet::Controls))
            .add_systems(Update, advance_time.in_set(SimulationSet::Advance));
    }
}

fn control_time(mut controls: MessageReader<TimeControl>, mut clock: ResMut<SimulationClock>) {
    for control in controls.read() {
        clock.set_paused(matches!(control, TimeControl::Pause));
    }
}

fn advance_time(
    time: Res<Time<Real>>,
    config: Res<GameConfig>,
    ships: Query<&ShipLocation>,
    mut clock: ResMut<SimulationClock>,
) {
    let next_arrival = ships.iter().filter_map(ShipLocation::arrival_unit).min();
    clock.advance(time.delta(), config.real_millis_per_unit, next_arrival);
}
