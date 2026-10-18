use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

use crate::content::{GameConfig, MapPosition};
use crate::ship::{Ship, ShipLocation};
use crate::simulation::{SimulationClock, SimulationSet};
use crate::trade::StationState;

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SelectedSystem(pub Entity);

#[derive(Component)]
struct SelectSystem(Entity);

#[derive(Component)]
struct MapStation;

#[derive(Component)]
struct ShipMapMarker;

pub(crate) fn spawn_star_map(
    commands: &mut Commands,
    config: &GameConfig,
    stations: &Query<(Entity, &StationState)>,
) -> Entity {
    let start = stations
        .iter()
        .find(|(_, station)| station.id == config.starting_station_id)
        .map(|(entity, _)| entity)
        .expect("validated content must contain the starting station");
    commands.insert_resource(SelectedSystem(start));

    let panel = commands
        .spawn((
            Node {
                width: px(390),
                max_width: percent(100),
                padding: px(16).all(),
                row_gap: px(10),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.055, 0.085, 0.13)),
        ))
        .id();
    let heading = commands
        .spawn((
            Text::new("STAR MAP / SELECT A SYSTEM"),
            text_font(20.0),
            TextColor(Color::srgb(0.7, 0.9, 1.0)),
        ))
        .id();
    let canvas = commands
        .spawn((
            Node {
                width: px(320),
                max_width: percent(100),
                height: px(320),
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(Color::srgb(0.025, 0.045, 0.075)),
        ))
        .id();

    for route in &config.routes {
        let Some(from) = config
            .stations
            .iter()
            .find(|station| station.id == route.from_station_id)
        else {
            continue;
        };
        let Some(to) = config
            .stations
            .iter()
            .find(|station| station.id == route.to_station_id)
        else {
            continue;
        };
        let x_delta = to.map_position.x - from.map_position.x;
        let y_delta = to.map_position.y - from.map_position.y;
        let length = x_delta.hypot(y_delta);
        let line = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: percent((from.map_position.x + to.map_position.x) * 50.0 - length * 50.0),
                    top: percent((from.map_position.y + to.map_position.y) * 50.0),
                    width: percent(length * 100.0),
                    height: px(2),
                    ..default()
                },
                UiTransform {
                    translation: Val2::new(Val::ZERO, px(-1.0)),
                    rotation: Rot2::radians(y_delta.atan2(x_delta)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.38, 0.52)),
            ))
            .id();
        commands.entity(canvas).add_child(line);
    }

    for station_config in &config.stations {
        let Some((entity, station)) = stations
            .iter()
            .find(|(_, station)| station.id == station_config.id)
        else {
            continue;
        };
        let button = commands
            .spawn((
                Button,
                Hovered::default(),
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(station_config.map_position.x * 100.0),
                    top: percent(station_config.map_position.y * 100.0),
                    min_width: px(128),
                    min_height: px(40),
                    padding: px(5).all(),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                UiTransform {
                    translation: Val2::percent(-50.0, -50.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.24, 0.31)),
                SelectSystem(entity),
                MapStation,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(station.name.clone()),
                    text_font(14.0),
                    TextColor(Color::WHITE),
                ));
            })
            .observe(select_system)
            .id();
        commands.entity(canvas).add_child(button);
    }

    let ship_marker = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: percent(0),
                top: percent(0),
                padding: px(3).all(),
                ..default()
            },
            UiTransform {
                translation: Val2::percent(-50.0, -145.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.8, 0.72, 0.3)),
            ShipMapMarker,
            children![(Text::new("SHIP"), text_font(12.0), TextColor(Color::BLACK),)],
        ))
        .id();
    commands.entity(canvas).add_child(ship_marker);
    commands.entity(panel).add_children(&[heading, canvas]);
    panel
}

pub(crate) struct StarMapPlugin;

impl Plugin for StarMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (refresh_selected_station, refresh_ship_marker).in_set(SimulationSet::Presentation),
        );
    }
}

fn select_system(
    activate: On<Activate>,
    systems: Query<&SelectSystem>,
    mut selected: ResMut<SelectedSystem>,
) {
    let Ok(system) = systems.get(activate.entity) else {
        return;
    };
    selected.0 = system.0;
}

fn refresh_selected_station(
    selected: Res<SelectedSystem>,
    mut stations: Query<(&SelectSystem, &mut BackgroundColor), With<MapStation>>,
) {
    for (system, mut color) in &mut stations {
        color.0 = match system.0 == selected.0 {
            true => Color::srgb(0.34, 0.55, 0.28),
            false => Color::srgb(0.12, 0.24, 0.31),
        };
    }
}

fn refresh_ship_marker(
    config: Res<GameConfig>,
    clock: Res<SimulationClock>,
    ship: Single<&ShipLocation, With<Ship>>,
    stations: Query<(Entity, &StationState)>,
    mut marker: Single<&mut Node, With<ShipMapMarker>>,
) {
    let Some(position) = (match **ship {
        ShipLocation::Docked { station } => station_position(station, &stations, &config),
        ShipLocation::InTransit {
            from,
            to,
            departed_at,
            arrives_at,
        } => {
            let Some(from) = station_position(from, &stations, &config) else {
                return;
            };
            let Some(to) = station_position(to, &stations, &config) else {
                return;
            };
            let duration = arrives_at.saturating_sub(departed_at).max(1);
            let elapsed = clock
                .elapsed_units
                .saturating_sub(departed_at)
                .min(duration);
            let progress = elapsed as f32 / duration as f32;
            Some(MapPosition {
                x: from.x + (to.x - from.x) * progress,
                y: from.y + (to.y - from.y) * progress,
            })
        }
    }) else {
        return;
    };
    marker.left = percent(position.x * 100.0);
    marker.top = percent(position.y * 100.0);
}

fn station_position(
    entity: Entity,
    stations: &Query<(Entity, &StationState)>,
    config: &GameConfig,
) -> Option<MapPosition> {
    let (_, station) = stations.get(entity).ok()?;
    config
        .stations
        .iter()
        .find(|configured| configured.id == station.id)
        .map(|configured| configured.map_position)
}

fn text_font(size: f32) -> TextFont {
    TextFont {
        font_size: FontSize::Px(size),
        ..default()
    }
}
