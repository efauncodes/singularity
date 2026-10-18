use std::time::Duration;

use bevy::input::InputPlugin;
use bevy::input_focus::InputFocusPlugin;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::{Activate, Button};
use singularity::app::GamePlugin;
use singularity::content::load_game_config;

fn text_entity(app: &mut App, value: &str) -> Entity {
    app.world_mut()
        .query::<(Entity, &Text)>()
        .iter(app.world())
        .find(|(_, text)| text.0 == value)
        .unwrap_or_else(|| panic!("missing UI text: {value}"))
        .0
}

fn button_entity(app: &mut App, label: &str) -> Entity {
    let text = text_entity(app, label);
    app.world().get::<ChildOf>(text).unwrap().parent()
}

fn destination_button(app: &mut App, station: &str) -> Entity {
    let title = text_entity(app, station);
    let panel = app.world().get::<ChildOf>(title).unwrap().parent();
    app.world()
        .get::<Children>(panel)
        .unwrap()
        .iter()
        .find(|&child| app.world().get::<Button>(child).is_some())
        .unwrap()
}

fn step(app: &mut App, duration: Duration) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(duration));
    app.update();
}

#[test]
fn production_ui_controls_time_travel_and_displays_arrival_in_the_same_update() {
    let config = load_game_config("assets/data/game.yaml").unwrap();
    let unit = Duration::from_millis(u64::from(config.real_millis_per_unit));
    let mut app = App::new();
    app.insert_resource(config)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
        .add_plugins(MinimalPlugins)
        .add_plugins((InputPlugin, InputFocusPlugin, GamePlugin));
    app.update();

    text_entity(&mut app, "Time: 0 units | PAUSED");
    let mars = destination_button(&mut app, "Mars Exchange");
    let europa = destination_button(&mut app, "Europa Exchange");
    destination_button(&mut app, "Ceres Exchange");
    assert!(app.world().entity(mars).contains::<InteractionDisabled>());
    assert!(!app.world().entity(europa).contains::<InteractionDisabled>());

    app.world_mut().trigger(Activate { entity: europa });
    step(&mut app, Duration::ZERO);
    assert!(app.world().entity(europa).contains::<InteractionDisabled>());
    let start = button_entity(&mut app, "Start time");
    let pause = button_entity(&mut app, "Pause time");
    app.world_mut().trigger(Activate { entity: start });
    step(&mut app, unit * 3);
    text_entity(&mut app, "Time: 3 units | RUNNING");
    app.world_mut().trigger(Activate { entity: pause });
    step(&mut app, Duration::from_secs(100));
    text_entity(&mut app, "Time: 3 units | PAUSED");

    app.world_mut().trigger(Activate { entity: start });
    step(&mut app, Duration::from_secs(100));
    text_entity(&mut app, "Time: 12 units | PAUSED");
    text_entity(&mut app, "Ship docked at Europa Exchange");
    text_entity(
        &mut app,
        "Ship arrived at Europa Exchange at time 12. Time paused.",
    );
    assert!(!app.world().entity(mars).contains::<InteractionDisabled>());
    assert!(app.world().entity(europa).contains::<InteractionDisabled>());
}
