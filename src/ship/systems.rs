use bevy::prelude::*;

use super::{ArrivalNotice, Ship, ShipArrived, ShipLocation, TravelRequest, TravelResult};
use crate::content::GameConfig;
use crate::simulation::{SimulationClock, SimulationSet};
use crate::trade::StationState;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TravelRequest>()
            .add_message::<ShipArrived>()
            .init_resource::<ArrivalNotice>()
            .init_resource::<TravelResult>()
            .add_systems(
                Update,
                process_travel_requests.in_set(SimulationSet::TravelOrders),
            )
            .add_systems(Update, finish_travel.in_set(SimulationSet::Arrivals))
            .add_systems(
                Update,
                announce_arrivals.in_set(SimulationSet::Notifications),
            );
    }
}

fn process_travel_requests(
    mut requests: MessageReader<TravelRequest>,
    mut ships: Query<&mut ShipLocation, With<Ship>>,
    stations: Query<&StationState>,
    config: Res<GameConfig>,
    clock: Res<SimulationClock>,
    mut result: ResMut<TravelResult>,
    mut notice: ResMut<ArrivalNotice>,
) {
    let Ok(mut location) = ships.single_mut() else {
        return;
    };

    for request in requests.read() {
        let ShipLocation::Docked { station: from } = *location else {
            result.0 = "The ship is already travelling. Wait for arrival.".to_string();
            continue;
        };
        if from == request.destination {
            result.0 = "The ship is already docked at that station.".to_string();
            continue;
        }
        let Ok(origin) = stations.get(from) else {
            result.0 = "The departure station is unavailable.".to_string();
            continue;
        };
        let Ok(destination) = stations.get(request.destination) else {
            result.0 = "That destination is unavailable.".to_string();
            continue;
        };
        let Some(duration) = config.travel_units(&origin.id, &destination.id) else {
            result.0 = "There is no direct route to that station.".to_string();
            continue;
        };
        let Some(arrives_at) = clock.elapsed_units.checked_add(u64::from(duration)) else {
            result.0 = "The journey would exceed the simulation time limit.".to_string();
            continue;
        };

        *location = ShipLocation::InTransit {
            from,
            to: request.destination,
            departed_at: clock.elapsed_units,
            arrives_at,
        };
        result.0 = format!(
            "Course set for {}: {duration} time units.",
            destination.name
        );
        notice.0 = None;
    }
}

fn finish_travel(
    clock: Res<SimulationClock>,
    mut ships: Query<(Entity, &mut ShipLocation), With<Ship>>,
    mut arrivals: MessageWriter<ShipArrived>,
) {
    for (ship, mut location) in &mut ships {
        let ShipLocation::InTransit { to, arrives_at, .. } = *location else {
            continue;
        };
        if clock.elapsed_units < arrives_at {
            continue;
        }

        *location = ShipLocation::Docked { station: to };
        arrivals.write(ShipArrived {
            ship,
            station: to,
            at_unit: arrives_at,
        });
    }
}

fn announce_arrivals(
    mut arrivals: MessageReader<ShipArrived>,
    stations: Query<&StationState>,
    mut clock: ResMut<SimulationClock>,
    mut notice: ResMut<ArrivalNotice>,
    mut result: ResMut<TravelResult>,
) {
    for arrival in arrivals.read() {
        clock.set_paused(true);
        let Ok(station) = stations.get(arrival.station) else {
            continue;
        };
        notice.0 = Some(format!(
            "Ship arrived at {} at time {}. Time paused.",
            station.name, arrival.at_unit
        ));
        result.0 = "Choose the next destination or resume time.".to_string();
    }
}
