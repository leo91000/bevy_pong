use bevy::prelude::*;
use lightyear::prelude::*;
use std::convert::Into;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
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

pub const REPLICATION_GROUP: ReplicationGroup = ReplicationGroup::new_id(1);

pub fn get_server_socket_addr() -> SocketAddr {
    SocketAddrV4::new(get_server_ip_addr(), get_server_port()).into()
}

pub fn get_server_port() -> u16 {
    option_env!("PONG_SERVER_PORT")
        .and_then(|v| v.parse().ok())
        .unwrap_or(32761)
}

pub fn get_server_ip_addr() -> Ipv4Addr {
    option_env!("PONG_SERVER_HOST")
        .and_then(|v| v.parse().ok())
        .unwrap_or(Ipv4Addr::UNSPECIFIED)
}
