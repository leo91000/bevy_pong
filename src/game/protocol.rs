use crate::game::shared::Border;
use crate::game::shared_const::BALL_RADIUS;
use avian2d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::client::{ComponentSyncMode, LerpFn};
use lightyear::prelude::*;
use lightyear::utils::bevy::TransformLinearInterpolation;
use std::random::{Random, RandomSource};

#[derive(Channel)]
pub struct Channel1;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect, Serialize, Deserialize)]
pub enum Action {
    Up,
    Down,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct PlayerId(pub ClientId);

// We need to serialize these components
#[derive(Component, Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Copy)]
#[require(
    RigidBody(|| RigidBody::Kinematic),
    Restitution(|| Restitution {
        coefficient: 1.,
        combine_rule: CoefficientCombine::Max,
    }),
    Friction(|| Friction::new(0.)),
)]
pub struct NetworkedPaddle {
    pub side: PaddleSide,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Copy)]
pub enum PaddleSide {
    Left,
    Right,
}

impl Random for PaddleSide {
    fn random(source: &mut (impl RandomSource + ?Sized)) -> Self {
        match bool::random(source) {
            true => PaddleSide::Left,
            false => PaddleSide::Right,
        }
    }
}

impl PaddleSide {
    pub fn other(&self) -> Self {
        match self {
            PaddleSide::Left => PaddleSide::Right,
            PaddleSide::Right => PaddleSide::Left,
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[require(
    RigidBody(|| RigidBody::Dynamic),
    Restitution(|| Restitution {
        coefficient: 1.,
        combine_rule: CoefficientCombine::Max,
    }),
    Friction(|| Friction::new(0.)),
    Collider(|| Collider::circle(BALL_RADIUS)),
    LinearVelocity(|| LinearVelocity(Vec2::new(200., 200.))),
    GravityScale(|| GravityScale(0.0)),
    LockedAxes(|| LockedAxes::ROTATION_LOCKED),
)]
pub struct NetworkedBall;

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_channel::<Channel1>(ChannelSettings {
            mode: ChannelMode::UnorderedUnreliable,
            ..default()
        });

        // Add Leafwing input plugin
        let lightyear_leafwing_plugin = LeafwingInputPlugin::<Action>::default();
        app.add_plugins(lightyear_leafwing_plugin);

        // Game markers
        app.register_component::<NetworkedBall>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full);
        app.register_component::<NetworkedPaddle>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full);
        app.register_component::<Border>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full);

        // Fully replicated, but not visual, so no need for lerp/corrections:
        app.register_component::<LinearVelocity>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full);

        app.register_component::<AngularVelocity>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full);

        app.register_component::<ExternalForce>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full);

        app.register_component::<ExternalImpulse>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full);

        app.register_component::<Transform>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full);

        app.register_component::<ComputedMass>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full);

        // Position and Rotation have a `correction_fn` set, which is used to smear rollback errors
        // over a few frames, just for the rendering part in postudpate.
        //
        // They also set `interpolation_fn` which is used by the VisualInterpolationPlugin to smooth
        // out rendering between fixedupdate ticks.
        app.register_component::<Position>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full)
            .add_interpolation_fn(lightyear::utils::avian2d::position::lerp)
            .add_correction_fn(lightyear::utils::avian2d::position::lerp);

        app.register_component::<Rotation>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full)
            .add_interpolation_fn(lightyear::utils::avian2d::rotation::lerp)
            .add_correction_fn(lightyear::utils::avian2d::rotation::lerp);

        // do not replicate Transform but make sure to register an interpolation function
        // for it so that we can do visual interpolation
        // (another option would be to replicate transform and not use Position/Rotation at all)
        app.add_interpolation::<Transform>(ComponentSyncMode::None);
        app.add_interpolation_fn::<Transform>(
            <TransformLinearInterpolation as LerpFn<Transform>>::lerp,
        );
    }
}
