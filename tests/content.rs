use singularity::content::{load_game_config, parse_game_config};

#[test]
fn production_market_content_loads_with_expected_starting_values() {
    let config =
        load_game_config("assets/data/game.yaml").expect("production YAML should be valid");

    assert_eq!(config.starting_credits, 300);
    assert_eq!(config.cargo_capacity, 100);
    assert_eq!(config.goods.len(), 3);
    assert_eq!(config.stations.len(), 2);
}

#[test]
fn duplicate_good_identifier_reports_source_and_identifier() {
    let source = include_str!("fixtures/duplicate_goods.yaml");
    let error = parse_game_config(source, "fixtures/duplicate_goods.yaml").unwrap_err();

    assert!(error.contains("fixtures/duplicate_goods.yaml"));
    assert!(error.contains("nutrient_gel"));
}

#[test]
fn zero_weight_reports_source_and_field() {
    let source = include_str!("fixtures/invalid_weight.yaml");
    let error = parse_game_config(source, "fixtures/invalid_weight.yaml").unwrap_err();

    assert!(error.contains("fixtures/invalid_weight.yaml"));
    assert!(error.contains("weight"));
}

#[test]
fn unknown_market_good_reports_station_and_identifier() {
    let source = include_str!("fixtures/unknown_good.yaml");
    let error = parse_game_config(source, "fixtures/unknown_good.yaml").unwrap_err();

    assert!(error.contains("fixtures/unknown_good.yaml"));
    assert!(error.contains("alpha"));
    assert!(error.contains("missing_good"));
}
