use avian2d::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow};
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
enum Action {
    Up,
    Down,
}

#[derive(Resource)]
struct GameArea {
    width: f32,
    height: f32,
}

#[derive(Resource)]
struct AccelerationTimer(Timer);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            InputManagerPlugin::<Action>::default(),
        ))
        .insert_resource(ClearColor(Color::srgb(1.0, 1.0, 1.0)))
        .insert_resource(AccelerationTimer(Timer::from_seconds(
            0.1,
            TimerMode::Repeating,
        )))
        .add_systems(
            Startup,
            (
                setup_game_area,
                setup_camera,
                (spawn_local_paddle, spawn_border, spawn_ball)
                    .after(setup_game_area)
                    .after(setup_camera),
            ),
        )
        .add_systems(Update, (move_paddle, check_collisions, accelerate_ball))
        .register_type::<Border>()
        .register_type::<BorderSide>()
        .register_type::<Ball>()
        .register_type::<Paddle>()
        .run();
}

fn setup_game_area(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.single();

    // Add some padding to keep elements away from the window edges
    let padding = 20.0;
    let game_width = window.width() - padding * 2.0;
    let game_height = window.height() - padding * 2.0;

    commands.insert_resource(GameArea {
        width: game_width,
        height: game_height,
    });
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Component, Reflect)]
#[require(
        BorderSide,
        RigidBody(|| RigidBody::Static),
        Collider,
        Restitution(|| Restitution {
            coefficient: 1.,
            combine_rule: CoefficientCombine::Max,
        }),
        Friction(|| Friction::new(0.)),
        Transform,
        Sprite,
)]
struct Border;

#[derive(Component, Default, Reflect)]
enum BorderSide {
    #[default]
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Component, Reflect)]
#[require(
        RigidBody(|| RigidBody::Dynamic),
        Restitution(|| Restitution {
            coefficient: 1.,
            combine_rule: CoefficientCombine::Max,
        }),
        Friction(|| Friction::new(0.)),
        Collider(|| Collider::circle(10.)),
        LinearVelocity(|| LinearVelocity(Vec2::new(200., 200.))),
        GravityScale(|| GravityScale(0.0)),
        Transform(|| Transform::from_xyz(0., 0., 0.)),
        LockedAxes(|| LockedAxes::ROTATION_LOCKED),
)]
struct Ball;

#[derive(Component, Reflect)]
#[require(
        RigidBody(|| RigidBody::Kinematic),
        Restitution(|| Restitution {
            coefficient: 1.,
            combine_rule: CoefficientCombine::Max,
        }),
        Friction(|| Friction::new(0.)),
        Collider,
        Transform,
        Sprite,
)]
struct Paddle;

fn spawn_local_paddle(mut commands: Commands, game_area: Res<GameArea>) {
    let input_map = InputMap::new([
        (Action::Up, KeyCode::ArrowUp),
        (Action::Down, KeyCode::ArrowDown),
    ]);

    let paddle_width = 10.0;
    let paddle_height = 50.0;
    let paddle_x = -(game_area.width / 2.0) + paddle_width * 2.0; // Position paddle near left border

    commands.spawn((
        Paddle,
        InputManagerBundle::with_map(input_map),
        Transform::from_xyz(paddle_x, 0., 0.),
        Collider::rectangle(paddle_width, paddle_height),
        Sprite {
            color: Color::srgb(0.25, 0.25, 0.25),
            custom_size: Some(Vec2::new(paddle_width, paddle_height)),
            ..Default::default()
        },
    ));
}

fn spawn_ball(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mesh = meshes.add(Circle::new(10.));
    let material = materials.add(Color::srgb(0.25, 0.25, 0.25));

    commands.spawn((Ball, Mesh2d(mesh), MeshMaterial2d(material)));
}

fn spawn_border(mut commands: Commands, game_area: Res<GameArea>) {
    let border_thickness = 20.0;
    let half_width = game_area.width / 2.0;
    let half_height = game_area.height / 2.0;

    // Adjust the height/width of vertical/horizontal borders to account for corners
    let vertical_height = game_area.height + border_thickness;
    let horizontal_width = game_area.width - border_thickness;

    // Left
    commands.spawn((
        Border,
        BorderSide::Left,
        Transform::from_xyz(-half_width, 0., 0.),
        Collider::rectangle(border_thickness, vertical_height),
        Sprite {
            color: Color::srgb(0.25, 0.25, 0.25),
            custom_size: Some(Vec2::new(border_thickness, vertical_height)),
            ..Default::default()
        },
    ));

    // Right
    commands.spawn((
        Border,
        BorderSide::Right,
        Transform::from_xyz(half_width, 0., 0.),
        Collider::rectangle(border_thickness, vertical_height),
        Sprite {
            color: Color::srgb(0.25, 0.25, 0.25),
            custom_size: Some(Vec2::new(border_thickness, vertical_height)),
            ..Default::default()
        },
    ));

    // Top
    commands.spawn((
        Border,
        BorderSide::Top,
        Transform::from_xyz(0., half_height, 0.),
        Collider::rectangle(horizontal_width, border_thickness),
        Sprite {
            color: Color::srgb(0.25, 0.25, 0.25),
            custom_size: Some(Vec2::new(horizontal_width, border_thickness)),
            ..Default::default()
        },
    ));

    // Bottom
    commands.spawn((
        Border,
        BorderSide::Bottom,
        Transform::from_xyz(0., -half_height, 0.),
        Collider::rectangle(horizontal_width, border_thickness),
        Sprite {
            color: Color::srgb(0.25, 0.25, 0.25),
            custom_size: Some(Vec2::new(horizontal_width, border_thickness)),
            ..Default::default()
        },
    ));
}

fn move_paddle(
    mut query: Query<(&ActionState<Action>, &mut Transform), With<Paddle>>,
    time: Res<Time>,
    game_area: Res<GameArea>,
) {
    for (action_state, mut transform) in query.iter_mut() {
        let mut direction = 0.;
        if action_state.pressed(&Action::Up) {
            direction += 1.;
        }
        if action_state.pressed(&Action::Down) {
            direction -= 1.;
        }

        let new_y = transform.translation.y + direction * time.delta_secs() * 300.;
        let max_y = game_area.height / 2.0 - 30.0; // Leave some space from borders
        transform.translation.y = new_y.clamp(-max_y, max_y);
    }
}

fn check_collisions(
    mut collision_events: EventReader<CollisionStarted>,
    mut ball_query: Query<&mut LinearVelocity, With<Ball>>,
) {
    for CollisionStarted(e1, e2) in collision_events.read() {
        if ball_query.get(*e1).is_ok() && ball_query.get(*e2).is_ok() {
            for mut ball_velocity in &mut ball_query {
                ball_velocity.0.x *= -1.;
            }
        }
    }
}

const ACCELERATION_FACTOR: f32 = 1.001;
fn accelerate_ball(
    time: Res<Time>,
    mut timer: ResMut<AccelerationTimer>,
    mut ball_query: Query<&mut LinearVelocity, With<Ball>>,
) {
    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        for mut velocity in &mut ball_query {
            velocity.0 *= ACCELERATION_FACTOR;
        }
    }
}
