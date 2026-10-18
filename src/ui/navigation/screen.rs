use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::Activate;

use super::super::trade_screen::{set_button_enabled, spawn_button, text_font};
use crate::ship::{ArrivalNotice, Ship, ShipLocation, TravelRequest, TravelResult};
use crate::simulation::{SimulationClock, SimulationSet, TimeControl};
use crate::trade::StationState;

pub(crate) struct NavigationUiPlugin;

impl Plugin for NavigationUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (refresh_navigation, refresh_navigation_buttons).in_set(SimulationSet::Presentation),
        );
    }
}

#[derive(Component)]
enum NavigationText {
    Clock,
    Location,
    TravelStatus,
    Arrival,
}

#[derive(Component)]
enum NavigationAction {
    Time(TimeControl),
    Travel(Entity),
}

pub(crate) fn spawn_navigation(commands: &mut Commands) -> Entity {
    let root = commands
        .spawn(Node {
            row_gap: px(8),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .id();
    for marker in [
        NavigationText::Clock,
        NavigationText::Location,
        NavigationText::TravelStatus,
        NavigationText::Arrival,
    ] {
        let text = commands
            .spawn((
                Text::new(""),
                text_font(18.0),
                TextColor(Color::srgb(0.8, 0.9, 0.8)),
                marker,
            ))
            .id();
        commands.entity(root).add_child(text);
    }

    let controls = commands
        .spawn(Node {
            column_gap: px(12),
            ..default()
        })
        .id();
    for (label, control) in [
        ("Start time", TimeControl::Start),
        ("Pause time", TimeControl::Pause),
    ] {
        let button = spawn_button(commands, label);
        commands
            .entity(button)
            .insert(NavigationAction::Time(control))
            .observe(queue_navigation);
        commands.entity(controls).add_child(button);
    }
    commands.entity(root).add_child(controls);
    root
}

pub(crate) fn spawn_travel_button(commands: &mut Commands, station: Entity) -> Entity {
    let button = spawn_button(commands, "Send ship here");
    commands
        .entity(button)
        .insert(NavigationAction::Travel(station))
        .observe(queue_navigation);
    button
}

fn queue_navigation(
    activate: On<Activate>,
    actions: Query<&NavigationAction, Without<InteractionDisabled>>,
    mut controls: MessageWriter<TimeControl>,
    mut requests: MessageWriter<TravelRequest>,
) {
    let Ok(action) = actions.get(activate.entity) else {
        return;
    };
    match action {
        NavigationAction::Time(control) => {
            controls.write(*control);
        }
        NavigationAction::Travel(destination) => {
            requests.write(TravelRequest {
                destination: *destination,
            });
        }
    }
}

fn refresh_navigation(
    clock: Res<SimulationClock>,
    ship: Single<&ShipLocation, With<Ship>>,
    stations: Query<&StationState>,
    result: Res<TravelResult>,
    notice: Res<ArrivalNotice>,
    mut texts: Query<(&mut Text, &NavigationText)>,
) {
    let time_state = match clock.is_paused() {
        true => "PAUSED",
        false => "RUNNING",
    };
    let station_name = |entity| {
        stations
            .get(entity)
            .map(|station| station.name.as_str())
            .unwrap_or("Unavailable station")
    };
    let location = match **ship {
        ShipLocation::Docked { station } => format!("Ship docked at {}", station_name(station)),
        ShipLocation::InTransit {
            from,
            to,
            departed_at,
            arrives_at,
        } => {
            let elapsed = clock.elapsed_units.saturating_sub(departed_at);
            let duration = arrives_at - departed_at;
            let progress = elapsed * 100 / duration;
            format!(
                "Ship: {} -> {} | {progress}% | {} units remaining",
                station_name(from),
                station_name(to),
                arrives_at.saturating_sub(clock.elapsed_units)
            )
        }
    };

    for (mut text, marker) in &mut texts {
        text.0 = match marker {
            NavigationText::Clock => format!("Time: {} units | {time_state}", clock.elapsed_units),
            NavigationText::Location => location.clone(),
            NavigationText::TravelStatus => result.0.clone(),
            NavigationText::Arrival => notice.0.clone().unwrap_or_default(),
        };
    }
}

fn refresh_navigation_buttons(
    mut commands: Commands,
    clock: Res<SimulationClock>,
    ship: Single<&ShipLocation, With<Ship>>,
    mut buttons: Query<(
        Entity,
        &NavigationAction,
        Has<InteractionDisabled>,
        &mut BackgroundColor,
    )>,
) {
    for (entity, action, disabled, mut color) in &mut buttons {
        let enabled = match action {
            NavigationAction::Time(TimeControl::Start) => clock.is_paused(),
            NavigationAction::Time(TimeControl::Pause) => !clock.is_paused(),
            NavigationAction::Travel(target) => {
                matches!(**ship, ShipLocation::Docked { station } if station != *target)
            }
        };
        set_button_enabled(&mut commands, entity, disabled, enabled, &mut color);
    }
}
