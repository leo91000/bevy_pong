use crate::game::protocol::{Action, NetworkedBall, NetworkedPaddle};
use crate::game::shared::{Border, BorderSide, GameArea};
use crate::game::shared_const::{
    server_private_key, shared_config, BALL_RADIUS, BORDER_THICKNESS, PADDLE_HEIGHT, PADDLE_WIDTH,
    PROTOCOL_ID,
};
use avian2d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::prelude::*;
use lightyear::client::networking::ClientCommands;
use lightyear::inputs::leafwing::input_buffer::InputBuffer;
use lightyear::prelude::client::{Predicted, Rollback};
use lightyear::prelude::*;
use lightyear::shared::replication::components::Controlled;
use std::random::random;

pub struct PongClientPlugin;

impl Plugin for PongClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins,
            client::ClientPlugins::new(get_client_config()),
        ));
        app.init_resource::<GameArea>();
        app.add_systems(Startup, (connect_to_server, spawn_camera));
        app.add_systems(FixedUpdate, handle_actions);
        app.add_systems(
            Update,
            (handle_new_border, handle_new_ball, handle_new_paddle),
        );
    }
}

fn get_client_config() -> client::ClientConfig {
    let netcode_config = client::NetcodeConfig::default();
    let server_addr = "127.0.0.1:32761".parse().unwrap();
    let io_config = client::IoConfig::from_transport(client::ClientTransport::WebTransportClient {
        server_addr,
        client_addr: "0.0.0.0:32761".parse().unwrap(),
    });
    let auth = client::Authentication::Manual {
        private_key: server_private_key(),
        server_addr,
        client_id: random(),
        protocol_id: PROTOCOL_ID,
    };

    let net_config: client::NetConfig = client::NetConfig::Netcode {
        config: netcode_config,
        io: io_config,
        auth,
    };

    client::ClientConfig {
        net: net_config,
        shared: shared_config(),
        ..default()
    }
}

fn connect_to_server(mut commands: Commands) {
    commands.connect_client();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

fn handle_actions(
    time: Res<Time>,
    mut query: Query<
        (&ActionState<Action>, &InputBuffer<Action>, &mut Transform),
        (With<Predicted>, With<NetworkedPaddle>),
    >,
    tick_manager: Res<TickManager>,
    rollback: Option<Res<Rollback>>,
) {
    let tick = rollback
        .as_ref()
        .map(|rb| tick_manager.tick_or_rollback_tick(rb))
        .unwrap_or_else(|| tick_manager.tick());

    for (action_state, input_buffer, mut transform) in &mut query {
        if input_buffer.get(tick).is_some() {
            apply_paddle_action(&time, action_state, &mut transform);
            continue;
        }

        if let Some((_, prev_action_state)) = input_buffer.get_last_with_tick() {
            apply_paddle_action(&time, prev_action_state, &mut transform);
        } else {
            apply_paddle_action(&time, action_state, &mut transform);
        }
    }
}

fn apply_paddle_action(
    time: &Res<Time>,
    action_state: &ActionState<Action>,
    transform: &mut Mut<Transform>,
) {
    let mut direction = 0.;
    if action_state.pressed(&Action::Up) {
        direction += 1.;
    }
    if action_state.pressed(&Action::Down) {
        direction -= 1.;
    }

    let new_y = transform.translation.y + direction * time.delta_secs() * 300.;
    transform.translation.y = new_y;
}

fn handle_new_paddle(
    mut commands: Commands,
    mut query: Query<(Entity, Has<Controlled>), (Added<Predicted>, With<NetworkedPaddle>)>,
) {
    for (entity, is_controlled) in &mut query {
        if is_controlled {
            info!("Adding InputMap to controlled and preducted entity {entity:?}");
            commands.entity(entity).insert(InputMap::new([
                (Action::Up, KeyCode::ArrowUp),
                (Action::Down, KeyCode::ArrowDown),
            ]));
        } else {
            info!("Remote character replicated to use: {entity:?}");
        }

        info!("Adding physics to paddle: {entity:?}");
        commands.entity(entity).insert((
            Collider::rectangle(PADDLE_WIDTH, PADDLE_HEIGHT),
            Sprite {
                color: Color::srgb(0.25, 0.25, 0.25),
                custom_size: Some(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
                ..Default::default()
            },
        ));
    }
}

fn handle_new_border(
    mut commands: Commands,
    game_area: Res<GameArea>,
    query: Query<(Entity, &Border), With<Predicted>>,
) {
    for (entity, border) in &query {
        // Adjust the height/width of vertical/horizontal borders to account for corners
        let vertical_height = game_area.height + BORDER_THICKNESS;
        let horizontal_width = game_area.width - BORDER_THICKNESS;

        let size = match border.side {
            BorderSide::Left => Vec2::new(BORDER_THICKNESS, vertical_height),
            BorderSide::Right => Vec2::new(BORDER_THICKNESS, vertical_height),
            BorderSide::Top => Vec2::new(horizontal_width, BORDER_THICKNESS),
            BorderSide::Bottom => Vec2::new(horizontal_width, BORDER_THICKNESS),
        };

        info!("Adding physics to border {border:?}: {entity:?}");
        commands.entity(entity).insert((
            Collider::rectangle(size.x, size.y),
            Sprite {
                color: Color::srgb(0.25, 0.25, 0.25),
                custom_size: Some(size),
                ..default()
            },
        ));
    }
}

fn handle_new_ball(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<Entity, (With<Predicted>, With<NetworkedBall>)>,
) {
    for entity in &query {
        let mesh = meshes.add(Circle::new(BALL_RADIUS));
        let material = materials.add(Color::srgb(0.25, 0.25, 0.25));

        info!("Adding mesh and materiel to ball: {entity:?}");
        commands
            .entity(entity)
            .insert((Mesh2d(mesh), MeshMaterial2d(material)));
    }
}
