#![feature(random)]

mod game;

use bevy::prelude::*;
use game::server::PongServerPlugin;

fn main() -> AppExit {
    App::new().add_plugins(PongServerPlugin::default()).run()
}
