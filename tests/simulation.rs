mod support;

use std::time::Duration;

use bevy::prelude::*;
use singularity::ship::{Ship, ShipLocation, TravelRequest};
use singularity::simulation::{SimulationClock, TimeControl};
use support::{setup_game, step, travel_to};

#[test]
fn game_starts_paused_with_one_ship_at_the_configured_station() {
    let mut game = setup_game(300);
    assert_eq!(game.stations.len(), 3);
    assert_eq!(
        game.app
            .world_mut()
            .query_filtered::<Entity, With<Ship>>()
            .iter(game.app.world())
            .count(),
        1
    );
    assert_eq!(
        game.app.world().get::<ShipLocation>(game.player),
        Some(&ShipLocation::Docked {
            station: game.stations[0]
        })
    );
    step(&mut game.app, Duration::from_secs(100));
    let clock = game.app.world().resource::<SimulationClock>();
    assert!(clock.is_paused());
    assert_eq!(clock.elapsed_units, 0);
}

#[test]
fn running_time_accumulates_fractional_units_and_manual_pause_freezes_it() {
    let mut game = setup_game(300);
    let quarter = Duration::from_millis(u64::from(game.config.real_millis_per_unit) / 4);
    game.app
        .world_mut()
        .resource_mut::<Messages<TimeControl>>()
        .write(TimeControl::Start);
    step(&mut game.app, quarter);
    assert_eq!(
        game.app.world().resource::<SimulationClock>().elapsed_units,
        0
    );
    game.app
        .world_mut()
        .resource_mut::<Messages<TimeControl>>()
        .write(TimeControl::Pause);
    step(&mut game.app, Duration::from_secs(100));
    assert_eq!(
        game.app.world().resource::<SimulationClock>().elapsed_units,
        0
    );
    game.app
        .world_mut()
        .resource_mut::<Messages<TimeControl>>()
        .write(TimeControl::Start);
    step(&mut game.app, quarter * 3);
    assert_eq!(
        game.app.world().resource::<SimulationClock>().elapsed_units,
        1
    );
    step(&mut game.app, quarter * 4);
    assert_eq!(
        game.app.world().resource::<SimulationClock>().elapsed_units,
        2
    );
}

#[test]
fn manual_pause_freezes_a_journey_and_resume_continues_from_the_same_unit() {
    let mut game = setup_game(300);
    let unit = Duration::from_millis(u64::from(game.config.real_millis_per_unit));
    game.app
        .world_mut()
        .resource_mut::<Messages<TravelRequest>>()
        .write(TravelRequest {
            destination: game.stations[1],
        });
    step(&mut game.app, Duration::ZERO);
    game.app
        .world_mut()
        .resource_mut::<Messages<TimeControl>>()
        .write(TimeControl::Start);
    step(&mut game.app, unit * 3);
    let location = *game.app.world().get::<ShipLocation>(game.player).unwrap();
    assert_eq!(
        game.app.world().resource::<SimulationClock>().elapsed_units,
        3
    );
    game.app
        .world_mut()
        .resource_mut::<Messages<TimeControl>>()
        .write(TimeControl::Pause);
    step(&mut game.app, Duration::from_secs(100));
    assert_eq!(
        game.app.world().resource::<SimulationClock>().elapsed_units,
        3
    );
    assert_eq!(
        game.app.world().get::<ShipLocation>(game.player),
        Some(&location)
    );
    game.app
        .world_mut()
        .resource_mut::<Messages<TimeControl>>()
        .write(TimeControl::Start);
    step(&mut game.app, unit);
    assert_eq!(
        game.app.world().resource::<SimulationClock>().elapsed_units,
        4
    );
}

#[test]
fn time_can_run_without_a_journey_after_automatic_pause() {
    let mut game = setup_game(300);
    travel_to(&mut game.app, game.stations[2]);
    let before = game.app.world().resource::<SimulationClock>().elapsed_units;
    game.app
        .world_mut()
        .resource_mut::<Messages<TimeControl>>()
        .write(TimeControl::Start);
    step(
        &mut game.app,
        Duration::from_millis(u64::from(game.config.real_millis_per_unit)),
    );
    let clock = game.app.world().resource::<SimulationClock>();
    assert_eq!(clock.elapsed_units, before + 1);
    assert!(!clock.is_paused());
}
