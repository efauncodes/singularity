use bevy::input_focus::{
    AutoFocus,
    tab_navigation::{TabGroup, TabIndex, TabNavigationPlugin},
};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui_widgets::{Activate, Button};

use crate::content::GameConfig;
use crate::trade::{Player, PlayerState, StationState, TradeDirection, TradeRequest, TradeResult};

pub struct TradeUiPlugin;

impl Plugin for TradeUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TabNavigationPlugin)
            .add_systems(Update, (refresh_trade_screen,));
    }
}

#[derive(Component)]
struct QuantityInput;

#[derive(Component)]
struct TradeAction {
    station: Entity,
    good_id: String,
    direction: TradeDirection,
}

#[derive(Component)]
enum TradeScreenText {
    MarketLine { station: Entity, good_id: String },
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
            BackgroundColor(Color::srgb(0.035, 0.055, 0.09)),
        ))
        .id();

    let title = spawn_text(
        &mut commands,
        "SINGULARITY / STATION MARKET",
        30.0,
        Color::srgb(0.7, 0.9, 1.0),
    );
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
            Text::new("Enter a quantity and trade between the two exchanges."),
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

    let stations_row = commands
        .spawn((Node {
            width: percent(100),
            column_gap: px(20),
            align_items: AlignItems::Start,
            ..default()
        },))
        .id();

    for (station_entity, station) in &stations {
        let station_panel = commands
            .spawn((
                Node {
                    width: percent(50),
                    padding: px(18).all(),
                    row_gap: px(10),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.07, 0.11, 0.16)),
            ))
            .id();
        let station_title = spawn_text(
            &mut commands,
            &station.name,
            22.0,
            Color::srgb(0.42, 0.82, 1.0),
        );
        commands.entity(station_panel).add_child(station_title);

        for good in &config.goods {
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
                        station: station_entity,
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
                station_entity,
                &good.id,
                TradeDirection::Buy,
                &format!("Buy ({buy_price} cr)"),
            );
            let sell_button = spawn_trade_button(
                &mut commands,
                station_entity,
                &good.id,
                TradeDirection::Sell,
                &format!("Sell ({sell_price} cr)"),
            );
            commands
                .entity(actions)
                .add_children(&[buy_button, sell_button]);
            commands
                .entity(station_panel)
                .add_children(&[market_line, actions]);
        }

        commands.entity(stations_row).add_child(station_panel);
    }

    commands
        .entity(root)
        .add_children(&[title, wallet, cargo, status, input_row, stations_row]);
}

fn spawn_trade_button(
    commands: &mut Commands,
    station: Entity,
    good_id: &str,
    direction: TradeDirection,
    label: &str,
) -> Entity {
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
            TradeAction {
                station,
                good_id: good_id.to_string(),
                direction,
            },
            children![(Text::new(label), text_font(15.0), TextColor(Color::WHITE),)],
        ))
        .observe(queue_trade)
        .id()
}

fn queue_trade(
    activate: On<Activate>,
    actions: Query<&TradeAction>,
    input: Single<&EditableText, With<QuantityInput>>,
    mut requests: MessageWriter<TradeRequest>,
    mut result: ResMut<TradeResult>,
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
        station: action.station,
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

    for (mut text, marker) in &mut texts {
        match marker {
            TradeScreenText::Wallet => text.0.clone_from(&wallet),
            TradeScreenText::Cargo => text.0.clone_from(&cargo),
            TradeScreenText::Status => text.0.clone_from(&result.0),
            TradeScreenText::MarketLine { station, good_id } => {
                let Ok(station) = stations.get(*station) else {
                    continue;
                };
                let Some(good) = config.goods.iter().find(|good| good.id == *good_id) else {
                    continue;
                };
                let stock = station.stock.get(&good.id).copied().unwrap_or_default();
                text.0 = format!(
                    "{} · weight {} each · stock {}",
                    good.name, good.weight, stock
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

fn text_font(size: f32) -> TextFont {
    TextFont {
        font_size: FontSize::Px(size),
        ..default()
    }
}
