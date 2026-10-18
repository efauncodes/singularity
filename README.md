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
- `starting_station_id` references the station where the player's single ship starts docked.
- `real_millis_per_unit` is a positive number of real milliseconds per simulation time unit.
- Each good has a unique `id`, a display `name`, and a positive `weight` per unit.
- Each station has a unique `id`, a display `name`, a normalized `map_position` (`x` and `y` from 0 to 1), and a `market` list. A station is the system's single dock point.
- Each market entry references a good and defines its initial `stock`, `player_buy_price` (the amount the player pays), and `player_sell_price` (the amount the station pays).
- Prices and stock are positive or zero as appropriate; the station's payout cannot exceed its sale price.
- Each `routes` entry connects `from_station_id` and `to_station_id` in both directions and defines a positive integer `travel_units` duration. Every pair of stations needs exactly one direct route; self routes, duplicate connections, and unknown station references are rejected.

The loader validates required fields, values, duplicate identifiers, map positions, and good references. Errors include the content file and affected field or identifier.

## Time and travel

The game starts paused at Mars Exchange. The star map shows each configured system and route. Selecting a system displays its current stock and player buy/sell prices independently of the ship's location. Trade is only available when the ship is docked at the selected system, including while time is paused. Sending the ship to the selected system sets its course without changing the pause state. The ship marker moves along its route during travel, and a journey cannot be redirected in transit.

Use **Start time** and **Pause time** to control the simulation clock. Ship progress and remaining travel units are derived from that clock. Arrival docks the ship, displays a separate notification, and automatically pauses at the exact arrival time, even if a frame spans more time than the journey. Resuming does not catch up time spent paused or unused time from the arrival frame.

## Manual verification

1. Run `cargo run --locked`. Expect three systems and their configured routes on the map, the ship at Mars Exchange, and time 0 / PAUSED.
2. Select Europa Exchange. Expect its market stock and buy/sell prices to appear while trade buttons stay disabled.
3. Click **Send ship to selected system**. Expect an active journey with 0% progress and time still paused; trade stays disabled and the ship marker follows the Mars–Europa route as time advances.
4. Click **Start time**, then **Pause time** partway through. Expect the clock and progress to advance and then remain fixed while paused.
5. Click **Start time** again. Expect arrival at Europa, a notification naming Europa, and an automatic pause at time 12. Europa trade becomes available.
6. Buy or sell goods. Expect the selected market stock, cargo, and credits to update. Select Ceres while docked at Europa and confirm its market is visible but trading is disabled until arrival there.
7. Resize the window to 1024x768 or 800x600. Use the mouse wheel or trackpad to scroll between the map, selected market, and time controls. Expect all actions to remain reachable.
