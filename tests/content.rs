use singularity::content::{load_game_config, parse_game_config};

#[test]
fn production_market_content_loads_with_expected_starting_values() {
    let config =
        load_game_config("assets/data/game.yaml").expect("production YAML should be valid");

    assert_eq!(config.starting_credits, 300);
    assert_eq!(config.cargo_capacity, 100);
    assert_eq!(config.goods.len(), 3);
    assert_eq!(config.stations.len(), 3);
    assert_eq!(config.starting_station_id, "mars_exchange");
    assert_eq!(config.real_millis_per_unit, 500);
    assert_eq!(config.routes.len(), 3);
    for from in &config.stations {
        for to in &config.stations {
            if from.id == to.id {
                continue;
            }
            assert!(
                config
                    .travel_units(&from.id, &to.id)
                    .is_some_and(|units| units > 0)
            );
        }
    }
}

#[test]
fn invalid_travel_content_reports_source_and_affected_field_or_identifier() {
    let fixture = include_str!("fixtures/travel.yaml");
    let cases = [
        (
            "starting_station_id: alpha",
            "starting_station_id: unknown",
            "starting_station_id",
        ),
        (
            "real_millis_per_unit: 500",
            "real_millis_per_unit: 0",
            "real_millis_per_unit",
        ),
        ("travel_units: 5", "travel_units: 0", "travel_units"),
        ("to_station_id: beta", "to_station_id: missing", "missing"),
        (
            "from_station_id: alpha",
            "from_station_id: missing",
            "missing",
        ),
        (
            "to_station_id: beta",
            "to_station_id: alpha",
            "different stations",
        ),
        ("starting_station_id: alpha\n", "", "starting_station_id"),
        ("real_millis_per_unit: 500\n", "", "real_millis_per_unit"),
        ("    travel_units: 5\n", "", "travel_units"),
    ];
    for (before, after, expected) in cases {
        let source = fixture.replace(before, after);
        let error = parse_game_config(&source, "fixtures/travel.yaml").unwrap_err();
        assert!(error.contains("fixtures/travel.yaml"), "{error}");
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn duplicate_routes_are_rejected_in_both_directions() {
    let fixture = include_str!("fixtures/travel.yaml");
    for (from, to) in [("alpha", "beta"), ("beta", "alpha")] {
        let source = format!(
            "{fixture}  - from_station_id: {from}\n    to_station_id: {to}\n    travel_units: 7\n"
        );
        let error = parse_game_config(&source, "duplicate_routes.yaml").unwrap_err();
        assert!(error.contains("duplicate_routes.yaml"));
        assert!(error.contains("duplicate route"));
        assert!(error.contains("alpha"));
        assert!(error.contains("beta"));
    }
}

#[test]
fn missing_direct_route_is_rejected_with_station_identifiers() {
    let fixture = include_str!("fixtures/travel.yaml");
    let source = fixture.replace(
        "routes:\n  - from_station_id: alpha\n    to_station_id: beta\n    travel_units: 5\n",
        "routes: []\n",
    );
    let error = parse_game_config(&source, "missing_route.yaml").unwrap_err();
    assert!(error.contains("missing_route.yaml"));
    assert!(error.contains("routes"));
    assert!(error.contains("alpha"));
    assert!(error.contains("beta"));
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
