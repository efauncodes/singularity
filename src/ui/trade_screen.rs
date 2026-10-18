use bevy::input_focus::{
    AutoFocus,
    tab_navigation::{TabGroup, TabIndex, TabNavigationPlugin},
};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::{Activate, Button, ScrollArea};

use super::navigation::{NavigationUiPlugin, spawn_navigation, spawn_travel_button};
use super::star_map::{SelectedSystem, StarMapPlugin, spawn_star_map};
use crate::content::GameConfig;
use crate::ship::{Ship, ShipLocation};
use crate::simulation::SimulationSet;
use crate::trade::{Player, PlayerState, StationState, TradeDirection, TradeRequest, TradeResult};

pub struct TradeUiPlugin;

impl Plugin for TradeUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((TabNavigationPlugin, NavigationUiPlugin, StarMapPlugin))
            .add_systems(
                Update,
                (
                    refresh_trade_screen,
                    refresh_trade_buttons,
                    refresh_trade_button_labels,
                )
                    .in_set(SimulationSet::Presentation),
            );
    }
}

#[derive(Component)]
struct QuantityInput;

#[derive(Component)]
struct TradeAction {
    good_id: String,
    direction: TradeDirection,
}

#[derive(Component)]
enum TradeScreenText {
    MarketTitle,
    MarketLine { good_id: String },
    Wallet,
    Cargo,
    Status,
}

pub fn setup_trade_ui(
    mut commands: Commands,
    config: Res<GameConfig>,
    stations: Query<(Entity, &StationState)>,
) {
    commands.spawn(Camera2d);
    let root = commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                padding: px(28).all(),
                row_gap: px(16),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollArea,
            BackgroundColor(Color::srgb(0.035, 0.055, 0.09)),
        ))
        .id();

    let title = spawn_text(
        &mut commands,
        "SINGULARITY / STAR MAP",
        30.0,
        Color::srgb(0.7, 0.9, 1.0),
    );
    let navigation = spawn_navigation(&mut commands);
    let wallet = commands
        .spawn((
            Text::new(""),
            text_font(22.0),
            TextColor(Color::srgb(0.95, 0.85, 0.48)),
            TradeScreenText::Wallet,
        ))
        .id();
    let cargo = commands
        .spawn((
            Text::new(""),
            text_font(18.0),
            TextColor(Color::srgb(0.7, 0.86, 0.96)),
            TradeScreenText::Cargo,
        ))
        .id();
    let status = commands
        .spawn((
            Text::new("Enter a quantity to trade at your docked station."),
            text_font(17.0),
            TextColor(Color::srgb(0.74, 0.85, 0.75)),
            TradeScreenText::Status,
        ))
        .id();

    let input_row = commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                column_gap: px(12),
                ..default()
            },
            TabGroup::new(0),
        ))
        .id();
    let input_label = spawn_text(&mut commands, "Quantity", 18.0, Color::WHITE);
    let input = commands
        .spawn((
            Node {
                width: px(150),
                height: px(42),
                padding: px(8).all(),
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.4, 0.58, 0.72)),
            BackgroundColor(Color::srgb(0.08, 0.13, 0.19)),
            EditableText {
                visible_width: Some(10.0),
                allow_newlines: false,
                max_characters: Some(9),
                ..default()
            },
            TextLayout::no_wrap(),
            text_font(20.0),
            TextColor(Color::WHITE),
            TextCursorStyle::default(),
            QuantityInput,
            AutoFocus,
            TabIndex(0),
        ))
        .id();
    commands
        .entity(input_row)
        .add_children(&[input_label, input]);

    let map_and_market = commands
        .spawn((Node {
            width: percent(100),
            column_gap: px(18),
            row_gap: px(18),
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Start,
            ..default()
        },))
        .id();

    let map = spawn_star_map(&mut commands, &config, &stations);
    let market_panel = commands
        .spawn((
            Node {
                flex_basis: px(340),
                flex_grow: 1.0,
                padding: px(18).all(),
                row_gap: px(12),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.07, 0.11, 0.16)),
        ))
        .id();
    let market_title = commands
        .spawn((
            Text::new(""),
            text_font(22.0),
            TextColor(Color::srgb(0.42, 0.82, 1.0)),
            TradeScreenText::MarketTitle,
        ))
        .id();
    commands.entity(market_panel).add_child(market_title);
    let travel_button = spawn_travel_button(&mut commands);
    commands.entity(market_panel).add_child(travel_button);

    for good in &config.goods {
        let Some(station) = stations
            .iter()
            .find(|(_, station)| station.id == config.starting_station_id)
            .map(|(_, station)| station)
        else {
            continue;
        };
        let Some(buy_price) = station.player_buy_prices.get(&good.id) else {
            continue;
        };
        let Some(sell_price) = station.player_sell_prices.get(&good.id) else {
            continue;
        };
        let market_line = commands
            .spawn((
                Text::new(""),
                text_font(16.0),
                TextColor(Color::srgb(0.87, 0.9, 0.92)),
                TradeScreenText::MarketLine {
                    good_id: good.id.clone(),
                },
            ))
            .id();
        let actions = commands
            .spawn((Node {
                column_gap: px(8),
                align_items: AlignItems::Center,
                ..default()
            },))
            .id();
        let buy_button = spawn_trade_button(
            &mut commands,
            &good.id,
            TradeDirection::Buy,
            &format!("Buy ({buy_price} cr)"),
        );
        let sell_button = spawn_trade_button(
            &mut commands,
            &good.id,
            TradeDirection::Sell,
            &format!("Sell ({sell_price} cr)"),
        );
        commands
            .entity(actions)
            .add_children(&[buy_button, sell_button]);
        commands
            .entity(market_panel)
            .add_children(&[market_line, actions]);
    }
    commands
        .entity(map_and_market)
        .add_children(&[map, market_panel]);

    commands.entity(root).add_children(&[
        title,
        navigation,
        wallet,
        cargo,
        status,
        input_row,
        map_and_market,
    ]);
}

fn spawn_trade_button(
    commands: &mut Commands,
    good_id: &str,
    direction: TradeDirection,
    label: &str,
) -> Entity {
    let button = spawn_button(commands, label);
    commands
        .entity(button)
        .insert(TradeAction {
            good_id: good_id.to_string(),
            direction,
        })
        .observe(queue_trade)
        .id()
}

pub(super) fn spawn_button(commands: &mut Commands, label: &str) -> Entity {
    commands
        .spawn((
            Button,
            Hovered::default(),
            Node {
                min_width: px(130),
                min_height: px(36),
                padding: px(6).all(),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.24, 0.31)),
            children![(Text::new(label), text_font(15.0), TextColor(Color::WHITE),)],
        ))
        .id()
}

fn queue_trade(
    activate: On<Activate>,
    actions: Query<&TradeAction, Without<InteractionDisabled>>,
    input: Single<&EditableText, With<QuantityInput>>,
    mut requests: MessageWriter<TradeRequest>,
    mut result: ResMut<TradeResult>,
    selected: Res<SelectedSystem>,
) {
    let Ok(action) = actions.get(activate.entity) else {
        return;
    };
    let Ok(quantity) = input.value().to_string().trim().parse::<u32>() else {
        result.0 = "Enter a whole number greater than zero.".to_string();
        return;
    };
    if quantity == 0 {
        result.0 = "Enter a whole number greater than zero.".to_string();
        return;
    }

    requests.write(TradeRequest {
        station: selected.0,
        good_id: action.good_id.clone(),
        quantity,
        direction: action.direction,
    });
}

fn refresh_trade_screen(
    config: Res<GameConfig>,
    player_query: Query<&PlayerState, With<Player>>,
    stations: Query<&StationState>,
    mut texts: Query<(&mut Text, &TradeScreenText)>,
    result: Res<TradeResult>,
    selected: Res<SelectedSystem>,
) {
    let Some(player) = player_query.iter().next() else {
        return;
    };
    let wallet = format!("Credits: {} cr", player.credits);
    let cargo = format!(
        "Cargo weight: {} / {} units",
        player.cargo_weight(&config.goods),
        player.cargo_capacity
    );
    let Ok(station) = stations.get(selected.0) else {
        return;
    };

    for (mut text, marker) in &mut texts {
        match marker {
            TradeScreenText::MarketTitle => text.0 = format!("MARKET / {}", station.name),
            TradeScreenText::Wallet => text.0.clone_from(&wallet),
            TradeScreenText::Cargo => text.0.clone_from(&cargo),
            TradeScreenText::Status => text.0.clone_from(&result.0),
            TradeScreenText::MarketLine { good_id } => {
                let Some(good) = config.goods.iter().find(|good| good.id == *good_id) else {
                    continue;
                };
                let stock = station.stock.get(&good.id).copied().unwrap_or_default();
                let buy_price = station
                    .player_buy_prices
                    .get(&good.id)
                    .copied()
                    .unwrap_or_default();
                let sell_price = station
                    .player_sell_prices
                    .get(&good.id)
                    .copied()
                    .unwrap_or_default();
                text.0 = format!(
                    "{} · stock {} · buy {} cr · sell {} cr",
                    good.name, stock, buy_price, sell_price
                );
            }
        }
    }
}

fn spawn_text(commands: &mut Commands, value: &str, size: f32, color: Color) -> Entity {
    commands
        .spawn((Text::new(value), text_font(size), TextColor(color)))
        .id()
}

pub(super) fn text_font(size: f32) -> TextFont {
    TextFont {
        font_size: FontSize::Px(size),
        ..default()
    }
}

fn refresh_trade_buttons(
    mut commands: Commands,
    ship: Single<&ShipLocation, With<Ship>>,
    selected: Res<SelectedSystem>,
    mut buttons: Query<(
        Entity,
        &TradeAction,
        Has<InteractionDisabled>,
        &mut BackgroundColor,
    )>,
) {
    for (entity, _, disabled, mut color) in &mut buttons {
        set_button_enabled(
            &mut commands,
            entity,
            disabled,
            ship.is_docked_at(selected.0),
            &mut color,
        );
    }
}

fn refresh_trade_button_labels(
    selected: Res<SelectedSystem>,
    stations: Query<&StationState>,
    buttons: Query<(&Children, &TradeAction)>,
    mut texts: Query<&mut Text>,
) {
    if !selected.is_changed() {
        return;
    }

    let Ok(station) = stations.get(selected.0) else {
        return;
    };
    for (children, action) in &buttons {
        let prices = match action.direction {
            TradeDirection::Buy => &station.player_buy_prices,
            TradeDirection::Sell => &station.player_sell_prices,
        };
        let price = prices.get(&action.good_id).copied().unwrap_or_default();
        let label = match action.direction {
            TradeDirection::Buy => format!("Buy ({price} cr)"),
            TradeDirection::Sell => format!("Sell ({price} cr)"),
        };
        for child in children {
            let Ok(mut text) = texts.get_mut(*child) else {
                continue;
            };
            text.0.clone_from(&label);
        }
    }
}

pub(super) fn set_button_enabled(
    commands: &mut Commands,
    entity: Entity,
    disabled: bool,
    enabled: bool,
    color: &mut BackgroundColor,
) {
    if enabled && disabled {
        commands.entity(entity).remove::<InteractionDisabled>();
    }
    if !enabled && !disabled {
        commands.entity(entity).insert(InteractionDisabled);
    }
    color.0 = match enabled {
        true => Color::srgb(0.12, 0.24, 0.31),
        false => Color::srgb(0.09, 0.11, 0.13),
    };
}
