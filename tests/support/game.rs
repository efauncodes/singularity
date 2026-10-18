use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use singularity::app::GameCorePlugin;
use singularity::content::{GameConfig, load_game_config};
use singularity::ship::{Ship, ShipLocation, TravelRequest};
use singularity::simulation::{SimulationClock, TimeControl};
use singularity::trade::{PlayerState, StationState};

pub struct TestGame {
    pub app: App,
    pub player: Entity,
    pub stations: Vec<Entity>,
    pub config: GameConfig,
}

pub fn setup_game(credits: u32) -> TestGame {
    let config = load_game_config("assets/data/game.yaml").expect("production YAML should load");
    let mut app = App::new();
    app.insert_resource(config.clone())
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
        .add_plugins(MinimalPlugins)
        .add_plugins(GameCorePlugin);
    app.update();
    let player = app
        .world_mut()
        .query_filtered::<Entity, With<Ship>>()
        .single(app.world())
        .unwrap();
    app.world_mut()
        .get_mut::<PlayerState>(player)
        .unwrap()
        .credits = credits;
    let stations = config
        .stations
        .iter()
        .map(|station_config| {
            app.world_mut()
                .query::<(Entity, &StationState)>()
                .iter(app.world())
                .find(|(_, station)| station.id == station_config.id)
                .unwrap()
                .0
        })
        .collect();
    TestGame {
        app,
        player,
        stations,
        config,
    }
}

pub fn step(app: &mut App, duration: Duration) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(duration));
    app.update();
}

pub fn travel_to(app: &mut App, destination: Entity) {
    app.world_mut()
        .resource_mut::<Messages<TravelRequest>>()
        .write(TravelRequest { destination });
    step(app, Duration::ZERO);
    let location = *app
        .world_mut()
        .query::<&ShipLocation>()
        .single(app.world())
        .unwrap();
    let ShipLocation::InTransit { arrives_at, .. } = location else {
        panic!("a valid travel request should launch the ship");
    };
    let remaining = arrives_at - app.world().resource::<SimulationClock>().elapsed_units;
    let millis_per_unit = u64::from(app.world().resource::<GameConfig>().real_millis_per_unit);
    app.world_mut()
        .resource_mut::<Messages<TimeControl>>()
        .write(TimeControl::Start);
    step(app, Duration::from_millis(remaining * millis_per_unit));
    assert_eq!(
        *app.world_mut()
            .query::<&ShipLocation>()
            .single(app.world())
            .unwrap(),
        ShipLocation::Docked {
            station: destination
        }
    );
    assert!(app.world().resource::<SimulationClock>().is_paused());
}
