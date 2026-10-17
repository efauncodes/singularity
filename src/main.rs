use std::process::ExitCode;

use bevy::prelude::*;
use singularity::app::GamePlugin;
use singularity::content::load_game_config;

fn main() -> ExitCode {
    let config = match load_game_config("assets/data/game.yaml") {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

    App::new()
        .insert_resource(config)
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePlugin)
        .run();

    ExitCode::SUCCESS
}
