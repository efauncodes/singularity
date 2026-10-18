use std::collections::HashSet;
use std::fs;
use std::path::Path;

use super::schema::GameConfig;

pub fn load_game_config(path: impl AsRef<Path>) -> Result<GameConfig, String> {
    let path = path.as_ref();
    let display_path = path.display().to_string();
    let source = fs::read_to_string(path)
        .map_err(|error| format!("{display_path}: could not read configuration: {error}"))?;
    let config: GameConfig = serde_saphyr::from_str(&source)
        .map_err(|error| format!("{display_path}: invalid YAML content: {error}"))?;
    validate_config(&config, &display_path)?;
    Ok(config)
}

pub fn parse_game_config(source: &str, source_name: &str) -> Result<GameConfig, String> {
    let config: GameConfig = serde_saphyr::from_str(source)
        .map_err(|error| format!("{source_name}: invalid YAML content: {error}"))?;
    validate_config(&config, source_name)?;
    Ok(config)
}

fn validate_config(config: &GameConfig, source: &str) -> Result<(), String> {
    if config.cargo_capacity == 0 {
        return Err(format!(
            "{source}: field `cargo_capacity` must be greater than zero"
        ));
    }

    if config.goods.is_empty() {
        return Err(format!(
            "{source}: field `goods` must contain at least one good"
        ));
    }

    if config.stations.len() != 2 {
        return Err(format!(
            "{source}: field `stations` must contain exactly two stations"
        ));
    }

    let mut good_ids = HashSet::new();
    for good in &config.goods {
        if good.id.trim().is_empty() {
            return Err(format!("{source}: good identifier must not be empty"));
        }
        if !good_ids.insert(good.id.as_str()) {
            return Err(format!("{source}: duplicate good identifier `{}`", good.id));
        }
        if good.name.trim().is_empty() {
            return Err(format!(
                "{source}: good `{}` field `name` must not be empty",
                good.id
            ));
        }
        if good.weight == 0 {
            return Err(format!(
                "{source}: good `{}` field `weight` must be greater than zero",
                good.id
            ));
        }
    }

    let mut station_ids = HashSet::new();
    for station in &config.stations {
        if station.id.trim().is_empty() {
            return Err(format!("{source}: station identifier must not be empty"));
        }
        if !station_ids.insert(station.id.as_str()) {
            return Err(format!(
                "{source}: duplicate station identifier `{}`",
                station.id
            ));
        }
        if station.name.trim().is_empty() {
            return Err(format!(
                "{source}: station `{}` field `name` must not be empty",
                station.id
            ));
        }

        let mut market_good_ids = HashSet::new();
        for entry in &station.market {
            if !good_ids.contains(entry.good_id.as_str()) {
                return Err(format!(
                    "{source}: station `{}` field `market.good_id` references unknown good `{}`",
                    station.id, entry.good_id
                ));
            }
            if !market_good_ids.insert(entry.good_id.as_str()) {
                return Err(format!(
                    "{source}: station `{}` has duplicate market entry for good `{}`",
                    station.id, entry.good_id
                ));
            }
            if entry.player_buy_price == 0 {
                return Err(format!(
                    "{source}: station `{}` good `{}` field `player_buy_price` must be greater than zero",
                    station.id, entry.good_id
                ));
            }
            if entry.player_sell_price == 0 {
                return Err(format!(
                    "{source}: station `{}` good `{}` field `player_sell_price` must be greater than zero",
                    station.id, entry.good_id
                ));
            }
            if entry.player_sell_price > entry.player_buy_price {
                return Err(format!(
                    "{source}: station `{}` good `{}` field `player_sell_price` must not exceed `player_buy_price`",
                    station.id, entry.good_id
                ));
            }
        }
        if market_good_ids.len() != good_ids.len() {
            return Err(format!(
                "{source}: station `{}` field `market` must define stock and prices for every good",
                station.id
            ));
        }
    }

    Ok(())
}
