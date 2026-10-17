use super::model::{PlayerState, StationState, TradeDirection, TradeRequest};
use crate::content::GoodConfig;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TradeError {
    InvalidQuantity,
    NotSoldHere,
    NotBoughtHere,
    InsufficientCredits,
    InsufficientStationStock,
    InsufficientCargo,
    CargoCapacityExceeded,
    StationStockOverflow,
}

impl std::fmt::Display for TradeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidQuantity => "Enter a whole number greater than zero.",
            Self::NotSoldHere => "This station does not sell that good.",
            Self::NotBoughtHere => "This station does not buy that good.",
            Self::InsufficientCredits => "Not enough credits.",
            Self::InsufficientStationStock => "The station does not have enough stock.",
            Self::InsufficientCargo => "You do not have enough of that good to sell.",
            Self::CargoCapacityExceeded => "That purchase exceeds your cargo capacity.",
            Self::StationStockOverflow => "The station cannot store any more of that good.",
        };
        formatter.write_str(message)
    }
}

pub fn execute_trade(
    player: &mut PlayerState,
    station: &mut StationState,
    goods: &[GoodConfig],
    request: &TradeRequest,
) -> Result<(), TradeError> {
    if request.quantity == 0 {
        return Err(TradeError::InvalidQuantity);
    }

    let Some(good) = goods.iter().find(|good| good.id == request.good_id) else {
        return Err(TradeError::NotSoldHere);
    };

    match request.direction {
        TradeDirection::Buy => buy(player, station, goods, good, request.quantity),
        TradeDirection::Sell => sell(player, station, good, request.quantity),
    }
}

fn buy(
    player: &mut PlayerState,
    station: &mut StationState,
    goods: &[GoodConfig],
    good: &GoodConfig,
    quantity: u32,
) -> Result<(), TradeError> {
    let Some(price) = station.player_buy_prices.get(&good.id).copied() else {
        return Err(TradeError::NotSoldHere);
    };
    let Some(stock) = station.stock.get(&good.id).copied() else {
        return Err(TradeError::NotSoldHere);
    };
    if stock < quantity {
        return Err(TradeError::InsufficientStationStock);
    }

    let total_cost = price
        .checked_mul(quantity)
        .ok_or(TradeError::InsufficientCredits)?;
    if player.credits < total_cost {
        return Err(TradeError::InsufficientCredits);
    }

    let current_weight = player.cargo_weight(goods);
    let added_weight = good
        .weight
        .checked_mul(quantity)
        .ok_or(TradeError::CargoCapacityExceeded)?;
    let resulting_weight = current_weight
        .checked_add(added_weight)
        .ok_or(TradeError::CargoCapacityExceeded)?;
    if resulting_weight > player.cargo_capacity {
        return Err(TradeError::CargoCapacityExceeded);
    }

    player.credits -= total_cost;
    *station.stock.entry(good.id.clone()).or_default() -= quantity;
    *player.cargo.entry(good.id.clone()).or_default() += quantity;
    Ok(())
}

fn sell(
    player: &mut PlayerState,
    station: &mut StationState,
    good: &GoodConfig,
    quantity: u32,
) -> Result<(), TradeError> {
    let Some(price) = station.player_sell_prices.get(&good.id).copied() else {
        return Err(TradeError::NotBoughtHere);
    };
    let Some(cargo) = player.cargo.get(&good.id).copied() else {
        return Err(TradeError::InsufficientCargo);
    };
    if cargo < quantity {
        return Err(TradeError::InsufficientCargo);
    }

    let payment = price
        .checked_mul(quantity)
        .ok_or(TradeError::InsufficientCredits)?;
    let resulting_credits = player
        .credits
        .checked_add(payment)
        .ok_or(TradeError::InsufficientCredits)?;
    let resulting_stock = station
        .stock
        .get(&good.id)
        .copied()
        .unwrap_or_default()
        .checked_add(quantity)
        .ok_or(TradeError::StationStockOverflow)?;
    player.credits = resulting_credits;
    *player.cargo.entry(good.id.clone()).or_default() -= quantity;
    station.stock.insert(good.id.clone(), resulting_stock);
    Ok(())
}
