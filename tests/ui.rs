use std::time::Duration;

use bevy::camera::RenderTarget;
use bevy::input::InputPlugin;
use bevy::input::mouse::MouseScrollUnit;
use bevy::input::touch::TouchPhase;
use bevy::input_focus::InputFocusPlugin;
use bevy::picking::{
    backend::HitData,
    events::{Pointer, Scroll},
    pointer::{Location, PointerId},
};
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::ui::{InteractionDisabled, ScrollPosition, UiGlobalTransform};
use bevy::ui_widgets::{Activate, Button};
use bevy::window::PrimaryWindow;
use bevy::winit::WinitPlugin;
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

fn node_bounds(app: &App, entity: Entity) -> Rect {
    let size = app.world().get::<ComputedNode>(entity).unwrap().size();
    let transform = app.world().get::<UiGlobalTransform>(entity).unwrap();
    Rect::from_center_size(transform.translation, size)
}

fn scroll_over(app: &mut App, entity: Entity, y: f32) {
    let camera = app
        .world_mut()
        .query_filtered::<Entity, With<Camera2d>>()
        .single(app.world())
        .unwrap();
    let window = app
        .world_mut()
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .single(app.world())
        .unwrap();
    let position = node_bounds(app, entity).center();
    app.world_mut().trigger(Pointer::new(
        PointerId::Mouse,
        Location {
            target: RenderTarget::default().normalize(Some(window)).unwrap(),
            position,
        },
        Scroll {
            unit: MouseScrollUnit::Pixel,
            x: 0.0,
            y,
            hit: HitData::new(camera, 0.0, None, None),
            phase: TouchPhase::Moved,
        },
        entity,
    ));
    app.update();
}

#[test]
fn wrapped_markets_scroll_into_view_in_small_windows() {
    for (width, height) in [(1024, 768), (800, 600)] {
        let mut app = App::new();
        app.insert_resource(load_game_config("assets/data/game.yaml").unwrap())
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
            .add_plugins(bevy::asset::AssetPlugin::default())
            .init_asset::<bevy::shader::Shader>()
            .add_plugins(
                DefaultPlugins
                    .build()
                    .disable::<bevy::asset::AssetPlugin>()
                    .disable::<WinitPlugin>()
                    .disable::<RenderPlugin>()
                    .disable::<bevy::log::LogPlugin>()
                    .disable::<bevy::gilrs::GilrsPlugin>()
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            resolution: (width, height).into(),
                            ..default()
                        }),
                        ..default()
                    }),
            )
            .add_plugins(GamePlugin)
            // RenderPlugin normally installs this camera update; it needs no GPU.
            .init_resource::<bevy::render::texture::ManualTextureViews>()
            .add_systems(
                PostUpdate,
                bevy::render::camera::camera_system.in_set(bevy::camera::CameraUpdateSystems),
            );
        app.finish();
        app.cleanup();
        // Let camera propagation, font loading and the real UI layout settle.
        for _ in 0..3 {
            app.update();
        }

        let root = app
            .world_mut()
            .query_filtered::<Entity, (With<Node>, Without<ChildOf>)>()
            .single(app.world())
            .unwrap();
        let target = button_entity(&mut app, "Sell (28 cr)");
        let visible_market = text_entity(&mut app, "Mars Exchange");
        let viewport = node_bounds(&app, root);
        assert_eq!(viewport.size(), Vec2::new(width as f32, height as f32));
        assert!(
            node_bounds(&app, target).max.y > viewport.max.y,
            "Ceres should initially overflow at {width}x{height}"
        );

        scroll_over(&mut app, visible_market, -10_000.0);
        assert!(app.world().get::<ScrollPosition>(root).unwrap().y > 0.0);
        let target_bounds = node_bounds(&app, target);
        assert!(
            target_bounds.min.y >= viewport.min.y,
            "Ceres action {target_bounds:?} must stay below the top edge of {viewport:?}"
        );
        assert!(
            target_bounds.max.y <= viewport.max.y,
            "Ceres action must be reachable by scrolling at {width}x{height}"
        );

        scroll_over(&mut app, target, 10_000.0);
        assert_eq!(app.world().get::<ScrollPosition>(root).unwrap().y, 0.0);
        let start = button_entity(&mut app, "Start time");
        assert!(node_bounds(&app, start).min.y >= viewport.min.y);
        assert!(node_bounds(&app, start).max.y <= viewport.max.y);
    }
}
