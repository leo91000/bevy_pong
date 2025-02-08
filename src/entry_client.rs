#![feature(random)]

mod game;

use bevy::prelude::*;
use game::client::PongClientPlugin;

fn main() -> AppExit {
    App::new().add_plugins(PongClientPlugin).run()
}
