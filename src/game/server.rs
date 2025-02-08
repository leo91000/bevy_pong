use crate::game::protocol::{
    Action, NetworkedBall, NetworkedPaddle, PaddleSide, PlayerId, ProtocolPlugin,
};
use crate::game::shared::{apply_paddle_action, Border, BorderSide, GameArea};
use crate::game::shared_const::{
    server_private_key, shared_config, BORDER_THICKNESS, PADDLE_HEIGHT, PADDLE_WIDTH, PROTOCOL_ID,
    REPLICATION_GROUP,
};
use avian2d::prelude::*;
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::scene::ScenePlugin;
use bevy::state::app::StatesPlugin;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::server::{ControlledBy, ServerCommands};
use lightyear::prelude::*;
use lightyear::server::config::ServerConfig;
use lightyear::server::plugin::ServerPlugins;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::random::random;

#[derive(Resource, Default)]
pub struct ServerState {
    connected_clients: Vec<ClientId>,
    ball_spawned: bool,
}

#[derive(Resource, Copy, Clone)]
pub struct PongServerConfig {
    pub port: u16,
}

impl Default for PongServerConfig {
    fn default() -> Self {
        Self { port: 32761 }
    }
}

#[derive(Default)]
pub struct PongServerPlugin {
    pub config: PongServerConfig,
}

fn webtransport_net_config(port: u16) -> server::NetConfig {
    let netcode_config = server::NetcodeConfig {
        private_key: server_private_key(),
        protocol_id: PROTOCOL_ID,
        ..default()
    };

    let ip: IpAddr = Ipv4Addr::UNSPECIFIED.into();
    let server_addr = SocketAddr::new(ip, port);

    let transport_config = server::ServerTransport::WebTransportServer {
        server_addr,
        certificate: server::Identity::self_signed(&["localhost", "127.0.0.1", "::1"]).unwrap(),
    };

    let io_config = server::IoConfig::from_transport(transport_config);

    server::NetConfig::Netcode {
        config: netcode_config,
        io: io_config,
    }
}

impl Plugin for PongServerPlugin {
    fn build(&self, app: &mut App) {
        let server_config = ServerConfig {
            shared: shared_config(),
            net: vec![webtransport_net_config(self.config.port)],
            ..default()
        };

        app.add_plugins((
            MinimalPlugins,
            LogPlugin {
                level: std::env::var("PONG_LOG_LEVEL")
                    .ok()
                    .and_then(|e| e.parse::<Level>().ok())
                    .unwrap_or(Level::INFO),
                ..default()
            },
            StatesPlugin,
            AssetPlugin::default(),
            ScenePlugin,
            HierarchyPlugin,
            PhysicsPlugins::default(),
            ServerPlugins::new(server_config),
            ProtocolPlugin,
        ))
        .insert_resource(self.config)
        .init_resource::<ServerState>()
        .init_resource::<GameArea>()
        .add_systems(Startup, (start_server, setup_server))
        .add_systems(FixedUpdate, handle_actions)
        .add_systems(Update, (handle_connections, handle_disconnections));
    }
}

fn start_server(mut commands: Commands, server_config: Res<PongServerConfig>) {
    info!("Server starting at 127.0.0.1:{0}", server_config.port);
    commands.start_server();
}

fn setup_server(commands: Commands, game_area: Res<GameArea>) {
    // Spawn borders
    info!("Adding borders");
    spawn_border(commands, game_area);
}

fn handle_actions(
    time: Res<Time>,
    mut query: Query<(&ActionState<Action>, &mut Transform), With<NetworkedPaddle>>,
) {
    for (action_state, mut transform) in &mut query {
        apply_paddle_action(&time, action_state, &mut transform);
    }
}

fn spawn_border(mut commands: Commands, game_area: Res<GameArea>) {
    let half_width = game_area.width / 2.0;
    let half_height = game_area.height / 2.0;

    // Adjust the height/width of vertical/horizontal borders to account for corners
    let vertical_height = game_area.height + BORDER_THICKNESS;
    let horizontal_width = game_area.width - BORDER_THICKNESS;

    // Left
    commands.spawn((
        Border {
            side: BorderSide::Left,
        },
        Transform::from_xyz(-half_width, 0., 0.),
        Collider::rectangle(BORDER_THICKNESS, vertical_height),
        server::Replicate {
            group: REPLICATION_GROUP,
            ..default()
        },
    ));

    // Right
    commands.spawn((
        Border {
            side: BorderSide::Right,
        },
        Transform::from_xyz(half_width, 0., 0.),
        Collider::rectangle(BORDER_THICKNESS, vertical_height),
        server::Replicate {
            group: REPLICATION_GROUP,
            ..default()
        },
    ));

    // Top
    commands.spawn((
        Border {
            side: BorderSide::Top,
        },
        Transform::from_xyz(0., half_height, 0.),
        Collider::rectangle(horizontal_width, BORDER_THICKNESS),
        server::Replicate {
            group: REPLICATION_GROUP,
            ..default()
        },
    ));

    // Bottom
    commands.spawn((
        Border {
            side: BorderSide::Bottom,
        },
        Transform::from_xyz(0., -half_height, 0.),
        Collider::rectangle(horizontal_width, BORDER_THICKNESS),
        server::Replicate {
            group: REPLICATION_GROUP,
            ..default()
        },
    ));
}

fn handle_connections(
    mut commands: Commands,
    mut connection_events: EventReader<server::ConnectEvent>,
    mut server_state: ResMut<ServerState>,
    game_area: Res<GameArea>,
    paddle: Query<&NetworkedPaddle>,
) {
    for event in connection_events.read() {
        let client_id = event.client_id;

        // Only allow 2 players
        if server_state.connected_clients.len() >= 2 {
            // Disconnect the client
            todo!("disconnect client whenever there is more than 2 players");
        }

        server_state.connected_clients.push(client_id);

        let existing_paddle = paddle.iter().next();
        // Determine paddle side based on connection order
        let side = if let Some(paddle) = existing_paddle {
            paddle.side.other()
        } else {
            random()
        };

        // Spawn paddle for the client
        spawn_networked_paddle(&mut commands, client_id, side, &game_area);

        // Spawn ball if this is the first client
        if server_state.connected_clients.len() == 1 && !server_state.ball_spawned {
            spawn_networked_ball(&mut commands);
            server_state.ball_spawned = true;
        }
    }
}

fn handle_disconnections(
    mut commands: Commands,
    mut disconnect_events: EventReader<server::DisconnectEvent>,
    mut server_state: ResMut<ServerState>,
    ball_query: Query<Entity, With<NetworkedBall>>,
) {
    for event in disconnect_events.read() {
        let client_id = event.client_id;

        // Remove client from connected clients
        if let Some(pos) = server_state
            .connected_clients
            .iter()
            .position(|&id| id == client_id)
        {
            server_state.connected_clients.remove(pos);
        }

        // If no clients left, despawn ball
        if server_state.connected_clients.is_empty() {
            for ball_entity in ball_query.iter() {
                commands.entity(ball_entity).despawn();
            }
            server_state.ball_spawned = false;
        }
    }
}

fn spawn_networked_paddle(
    commands: &mut Commands,
    client_id: ClientId,
    side: PaddleSide,
    game_area: &GameArea,
) {
    let paddle_x = match side {
        PaddleSide::Left => -(game_area.width / 2.0) + PADDLE_HEIGHT,
        PaddleSide::Right => (game_area.width / 2.0) - PADDLE_HEIGHT,
    };

    commands.spawn((
        NetworkedPaddle { side },
        PlayerId(client_id),
        Transform::from_xyz(paddle_x, 0., 0.),
        Collider::rectangle(PADDLE_WIDTH, PADDLE_HEIGHT),
        server::Replicate {
            controlled_by: ControlledBy {
                target: NetworkTarget::Single(client_id),
                ..default()
            },
            group: REPLICATION_GROUP,
            ..default()
        },
        ActionState::<Action>::default(),
    ));
}

fn spawn_networked_ball(commands: &mut Commands) {
    commands.spawn((
        NetworkedBall,
        server::Replicate {
            group: REPLICATION_GROUP,
            ..default()
        },
    ));
}
