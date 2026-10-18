mod loader;
mod schema;

pub use loader::{load_game_config, parse_game_config};
pub use schema::{GameConfig, GoodConfig, MarketEntry, RouteConfig, StationConfig};
