# Singularity

A space-trading economic simulation about merchants pursuing personal singularity through hypercapitalism.

## Run

```sh
cargo run --locked
```

## Market content

The initial game data is in `assets/data/game.yaml`.

- `starting_credits` is the player's initial wallet balance.
- `cargo_capacity` is the maximum cargo weight.
- Each good has a unique `id`, a display `name`, and a positive `weight` per unit.
- Each station has a unique `id`, a display `name`, and a `market` list.
- Each market entry references a good and defines its initial `stock`, `player_buy_price` (the amount the player pays), and `player_sell_price` (the amount the station pays).
- Prices and stock are positive or zero as appropriate; the station's payout cannot exceed its sale price.

The loader validates required fields, values, duplicate identifiers, and good references. Errors include the content file and affected field or identifier.
