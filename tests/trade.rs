mod support;

use std::time::Duration;

use bevy::prelude::*;
use singularity::content::{GameConfig, load_game_config};
use singularity::ship::{ArrivalNotice, TravelRequest};
use singularity::trade::{PlayerState, StationState, TradeDirection, TradeRequest, TradeResult};
use support::{setup_game, step, travel_to};

fn setup_app(credits: u32) -> (App, Entity, Entity, Entity, GameConfig) {
    let game = setup_game(credits);
    (
        game.app,
        game.player,
        game.stations[0],
        game.stations[1],
        game.config,
    )
}

fn send_trade(
    app: &mut App,
    station: Entity,
    good_id: &str,
    quantity: u32,
    direction: TradeDirection,
) {
    app.world_mut()
        .resource_mut::<Messages<TradeRequest>>()
        .write(TradeRequest {
            station,
            good_id: good_id.to_string(),
            quantity,
            direction,
        });
    app.update();
}

#[test]
fn successful_purchase_updates_wallet_station_stock_and_cargo() {
    let (mut app, player, alpha, _, _) = setup_app(300);

    send_trade(&mut app, alpha, "reactor_cells", 10, TradeDirection::Buy);

    let state = app.world().get::<PlayerState>(player).unwrap();
    let station = app.world().get::<StationState>(alpha).unwrap();
    assert_eq!(state.credits, 100);
    assert_eq!(state.cargo.get("reactor_cells"), Some(&10));
    assert_eq!(
        state.cargo_weight(&load_game_config("assets/data/game.yaml").unwrap().goods),
        100
    );
    assert_eq!(station.stock.get("reactor_cells"), Some(&5));
}

#[test]
fn successful_sale_updates_wallet_cargo_and_station_stock() {
    let (mut app, player, alpha, beta, config) = setup_app(300);
    send_trade(&mut app, alpha, "reactor_cells", 10, TradeDirection::Buy);

    travel_to(&mut app, beta);
    send_trade(&mut app, beta, "reactor_cells", 10, TradeDirection::Sell);

    let state = app.world().get::<PlayerState>(player).unwrap();
    let station = app.world().get::<StationState>(beta).unwrap();
    assert_eq!(state.credits, 450);
    assert_eq!(
        app.world().resource::<TradeResult>().0,
        "Trade complete: 10 x reactor_cells."
    );
    assert_eq!(state.cargo.get("reactor_cells"), Some(&0));
    assert_eq!(state.cargo_weight(&config.goods), 0);
    assert_eq!(station.stock.get("reactor_cells"), Some(&20));
}

#[test]
fn insufficient_credits_leave_transaction_state_unchanged() {
    let (mut app, player, _, beta, _) = setup_app(300);
    travel_to(&mut app, beta);
    let before_player = app.world().get::<PlayerState>(player).unwrap().clone();
    let before_station = app.world().get::<StationState>(beta).unwrap().clone();

    send_trade(&mut app, beta, "reactor_cells", 8, TradeDirection::Buy);

    assert_eq!(
        app.world().get::<PlayerState>(player).unwrap(),
        &before_player
    );
    assert_eq!(
        app.world().get::<StationState>(beta).unwrap().stock,
        before_station.stock
    );
    assert_eq!(
        app.world().resource::<TradeResult>().0,
        "Not enough credits."
    );
}

#[test]
fn insufficient_station_stock_leaves_transaction_state_unchanged() {
    let (mut app, player, alpha, _, _) = setup_app(3000);
    let before_player = app.world().get::<PlayerState>(player).unwrap().clone();
    let before_station = app.world().get::<StationState>(alpha).unwrap().clone();

    send_trade(&mut app, alpha, "nutrient_gel", 101, TradeDirection::Buy);

    assert_eq!(
        app.world().get::<PlayerState>(player).unwrap(),
        &before_player
    );
    assert_eq!(
        app.world().get::<StationState>(alpha).unwrap().stock,
        before_station.stock
    );
    assert_eq!(
        app.world().resource::<TradeResult>().0,
        "The station does not have enough stock."
    );
}

#[test]
fn insufficient_player_cargo_leaves_transaction_state_unchanged() {
    let (mut app, player, _, beta, _) = setup_app(300);
    travel_to(&mut app, beta);
    let before_player = app.world().get::<PlayerState>(player).unwrap().clone();
    let before_station = app.world().get::<StationState>(beta).unwrap().clone();

    send_trade(&mut app, beta, "nutrient_gel", 1, TradeDirection::Sell);

    assert_eq!(
        app.world().get::<PlayerState>(player).unwrap(),
        &before_player
    );
    assert_eq!(
        app.world().get::<StationState>(beta).unwrap().stock,
        before_station.stock
    );
    assert_eq!(
        app.world().resource::<TradeResult>().0,
        "You do not have enough of that good to sell."
    );
}

#[test]
fn remote_and_in_transit_trades_are_rejected_without_changing_inventory() {
    let (mut app, player, alpha, beta, _) = setup_app(300);
    let before_player = app.world().get::<PlayerState>(player).unwrap().clone();
    let before_alpha = app
        .world()
        .get::<StationState>(alpha)
        .unwrap()
        .stock
        .clone();
    let before_beta = app.world().get::<StationState>(beta).unwrap().stock.clone();
    send_trade(&mut app, beta, "nutrient_gel", 1, TradeDirection::Buy);
    assert_eq!(
        app.world().resource::<TradeResult>().0,
        "Dock at that station before trading."
    );
    app.world_mut()
        .resource_mut::<Messages<TravelRequest>>()
        .write(TravelRequest { destination: beta });
    step(&mut app, Duration::ZERO);
    for station in [alpha, beta] {
        send_trade(&mut app, station, "nutrient_gel", 1, TradeDirection::Buy);
        send_trade(&mut app, station, "nutrient_gel", 1, TradeDirection::Sell);
        assert_eq!(
            app.world().resource::<TradeResult>().0,
            "Dock at that station before trading."
        );
    }
    assert_eq!(
        app.world().get::<PlayerState>(player).unwrap(),
        &before_player
    );
    assert_eq!(
        app.world().get::<StationState>(alpha).unwrap().stock,
        before_alpha
    );
    assert_eq!(
        app.world().get::<StationState>(beta).unwrap().stock,
        before_beta
    );
}

#[test]
fn arrival_notification_survives_trade_feedback() {
    let (mut app, _, _, beta, _) = setup_app(300);
    travel_to(&mut app, beta);
    let notice = app.world().resource::<ArrivalNotice>().0.clone();
    send_trade(&mut app, beta, "reactor_cells", 10, TradeDirection::Buy);
    assert_eq!(
        app.world().resource::<TradeResult>().0,
        "Not enough credits."
    );
    assert_eq!(app.world().resource::<ArrivalNotice>().0, notice);
}

#[test]
fn purchase_over_cargo_capacity_is_rejected_without_mutation() {
    let (mut app, player, alpha, _, _) = setup_app(3000);
    let before_player = app.world().get::<PlayerState>(player).unwrap().clone();
    let before_station = app.world().get::<StationState>(alpha).unwrap().clone();

    send_trade(&mut app, alpha, "reactor_cells", 11, TradeDirection::Buy);

    assert_eq!(
        app.world().get::<PlayerState>(player).unwrap(),
        &before_player
    );
    assert_eq!(
        app.world().get::<StationState>(alpha).unwrap().stock,
        before_station.stock
    );
    assert_eq!(
        app.world().resource::<TradeResult>().0,
        "That purchase exceeds your cargo capacity."
    );
}

#[test]
fn cargo_capacity_counts_different_goods_together() {
    let (mut app, player, alpha, _, config) = setup_app(1000);

    send_trade(&mut app, alpha, "reactor_cells", 9, TradeDirection::Buy);
    send_trade(&mut app, alpha, "refined_alloy", 3, TradeDirection::Buy);

    let state = app.world().get::<PlayerState>(player).unwrap();
    assert_eq!(state.cargo.get("reactor_cells"), Some(&9));
    assert_eq!(state.cargo.get("refined_alloy"), None);
    assert_eq!(state.cargo_weight(&config.goods), 90);
    assert_eq!(
        app.world().resource::<TradeResult>().0,
        "That purchase exceeds your cargo capacity."
    );
}

#[test]
fn starting_balance_allows_a_direct_cargo_limit_rejection() {
    let (mut app, player, alpha, _, config) = setup_app(300);
    send_trade(&mut app, alpha, "reactor_cells", 10, TradeDirection::Buy);
    let player_after_fill = app.world().get::<PlayerState>(player).unwrap().clone();
    let station_after_fill = app.world().get::<StationState>(alpha).unwrap().clone();

    send_trade(&mut app, alpha, "nutrient_gel", 1, TradeDirection::Buy);

    let final_player = app.world().get::<PlayerState>(player).unwrap();
    let final_station = app.world().get::<StationState>(alpha).unwrap();
    assert_eq!(final_player, &player_after_fill);
    assert_eq!(final_station.stock, station_after_fill.stock);
    assert_eq!(final_player.credits, 100);
    assert_eq!(final_player.cargo_weight(&config.goods), 100);
    assert_eq!(
        app.world().resource::<TradeResult>().0,
        "That purchase exceeds your cargo capacity."
    );
}
