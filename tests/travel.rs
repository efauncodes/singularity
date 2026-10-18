mod support;

use std::time::Duration;

use bevy::ecs::message::MessageCursor;
use bevy::prelude::*;
use singularity::ship::{ArrivalNotice, ShipArrived, ShipLocation, TravelRequest, TravelResult};
use singularity::simulation::{SimulationClock, TimeControl};
use support::{setup_game, step, travel_to};

#[test]
fn a_travel_order_can_be_issued_while_paused_without_starting_time() {
    let mut game = setup_game(300);
    game.app
        .world_mut()
        .resource_mut::<Messages<TravelRequest>>()
        .write(TravelRequest {
            destination: game.stations[1],
        });
    step(&mut game.app, Duration::from_secs(100));
    let duration = game
        .config
        .travel_units(&game.config.stations[0].id, &game.config.stations[1].id)
        .unwrap();
    assert_eq!(
        game.app.world().get::<ShipLocation>(game.player),
        Some(&ShipLocation::InTransit {
            from: game.stations[0],
            to: game.stations[1],
            departed_at: 0,
            arrives_at: u64::from(duration),
        })
    );
    let clock = game.app.world().resource::<SimulationClock>();
    assert!(clock.is_paused());
    assert_eq!(clock.elapsed_units, 0);
}

#[test]
fn invalid_destinations_and_rerouting_leave_the_journey_unchanged() {
    let mut game = setup_game(300);
    let unavailable = game.app.world_mut().spawn_empty().id();
    for target in [game.stations[0], unavailable] {
        let before = *game.app.world().get::<ShipLocation>(game.player).unwrap();
        game.app
            .world_mut()
            .resource_mut::<Messages<TravelRequest>>()
            .write(TravelRequest {
                destination: target,
            });
        step(&mut game.app, Duration::ZERO);
        assert_eq!(
            game.app.world().get::<ShipLocation>(game.player),
            Some(&before)
        );
    }
    game.app
        .world_mut()
        .resource_mut::<Messages<TravelRequest>>()
        .write(TravelRequest {
            destination: game.stations[1],
        });
    step(&mut game.app, Duration::ZERO);
    let before = *game.app.world().get::<ShipLocation>(game.player).unwrap();
    game.app
        .world_mut()
        .resource_mut::<Messages<TravelRequest>>()
        .write(TravelRequest {
            destination: game.stations[2],
        });
    step(&mut game.app, Duration::ZERO);
    assert_eq!(
        game.app.world().get::<ShipLocation>(game.player),
        Some(&before)
    );
    assert!(
        game.app
            .world()
            .resource::<TravelResult>()
            .0
            .contains("already travelling")
    );
}

#[test]
fn arrival_stops_exactly_once_and_discards_excess_frame_time() {
    let mut game = setup_game(300);
    let duration = u64::from(
        game.config
            .travel_units(&game.config.stations[0].id, &game.config.stations[1].id)
            .unwrap(),
    );
    let mut arrivals = MessageCursor::<ShipArrived>::default();
    game.app
        .world_mut()
        .resource_mut::<Messages<TravelRequest>>()
        .write(TravelRequest {
            destination: game.stations[1],
        });
    game.app
        .world_mut()
        .resource_mut::<Messages<TimeControl>>()
        .write(TimeControl::Start);
    step(&mut game.app, Duration::from_secs(100));

    assert_eq!(
        game.app.world().get::<ShipLocation>(game.player),
        Some(&ShipLocation::Docked {
            station: game.stations[1]
        })
    );
    let clock = game.app.world().resource::<SimulationClock>();
    assert_eq!(clock.elapsed_units, duration);
    assert!(clock.is_paused());
    let events = arrivals
        .read(game.app.world().resource::<Messages<ShipArrived>>())
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(
        events,
        vec![ShipArrived {
            ship: game.player,
            station: game.stations[1],
            at_unit: duration
        }]
    );
    let notice = game.app.world().resource::<ArrivalNotice>().0.clone();
    assert!(
        notice
            .as_ref()
            .unwrap()
            .contains(&game.config.stations[1].name)
    );
    step(&mut game.app, Duration::from_secs(100));
    assert_eq!(
        game.app.world().resource::<SimulationClock>().elapsed_units,
        duration
    );
    assert_eq!(
        arrivals
            .read(game.app.world().resource::<Messages<ShipArrived>>())
            .count(),
        0
    );
    assert_eq!(game.app.world().resource::<ArrivalNotice>().0, notice);
    game.app
        .world_mut()
        .resource_mut::<Messages<TimeControl>>()
        .write(TimeControl::Start);
    step(&mut game.app, Duration::ZERO);
    assert_eq!(
        game.app.world().resource::<SimulationClock>().elapsed_units,
        duration
    );
    step(
        &mut game.app,
        Duration::from_millis(u64::from(game.config.real_millis_per_unit)),
    );
    assert_eq!(
        game.app.world().resource::<SimulationClock>().elapsed_units,
        duration + 1
    );
}

#[test]
fn every_station_pair_can_be_travelled_in_both_directions_repeatedly() {
    let mut game = setup_game(300);
    let mut origin = 0;
    let mut expected_time = 0;
    for destination in [1, 2, 0, 2, 1, 0] {
        expected_time += u64::from(
            game.config
                .travel_units(
                    &game.config.stations[origin].id,
                    &game.config.stations[destination].id,
                )
                .unwrap(),
        );
        travel_to(&mut game.app, game.stations[destination]);
        assert_eq!(
            game.app.world().resource::<SimulationClock>().elapsed_units,
            expected_time
        );
        origin = destination;
    }
}
