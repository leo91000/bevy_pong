#![feature(random)]

mod game;

use bevy::prelude::*;
use game::server::ServerPlugin;

fn main() -> AppExit {
    App::new().add_plugins(ServerPlugin::default()).run()
}
