use bevy::prelude::*;
use lightyear::prelude::*;
use std::time::Duration;

pub fn server_private_key() -> Key {
    [
        0x7b, 0xf4, 0x23, 0x9a, 0xd5, 0x8c, 0xe1, 0x4f, 0x3b, 0x92, 0x16, 0x5d, 0xa8, 0xc7, 0x0e,
        0x44, 0x6f, 0xb3, 0x2c, 0x85, 0xd9, 0x1e, 0x57, 0xaa, 0x94, 0x0d, 0x36, 0x8f, 0xc2, 0x4b,
        0x71, 0xe5,
    ]
}

pub const TICKS: u64 = 128;

pub fn shared_config() -> SharedConfig {
    let tick_duration = Duration::from_millis(1000 / TICKS);

    SharedConfig {
        mode: Mode::Separate,
        tick: TickConfig { tick_duration },
        ..default()
    }
}

pub const PADDLE_WIDTH: f32 = 10.;
pub const PADDLE_HEIGHT: f32 = 50.;

pub const BORDER_THICKNESS: f32 = 20.;

pub const BALL_RADIUS: f32 = 10.;

pub const PROTOCOL_ID: u64 = 0;
