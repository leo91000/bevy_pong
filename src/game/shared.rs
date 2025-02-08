use avian2d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource)]
pub struct GameArea {
    pub width: f32,
    pub height: f32,
}

impl Default for GameArea {
    fn default() -> Self {
        Self {
            width: 600.,
            height: 400.,
        }
    }
}

#[derive(Component, Reflect, Serialize, Deserialize, Debug, Eq, PartialEq, Copy, Clone)]
#[require(
    BorderSide,
    RigidBody(|| RigidBody::Static),
    Collider,
    Restitution(|| Restitution {
            coefficient: 1.,
            combine_rule: CoefficientCombine::Max,
        }),
    Friction(|| Friction::new(0.)),
)]
pub struct Border {
    pub side: BorderSide,
}

#[derive(
    Component, Default, Reflect, Serialize, Deserialize, Debug, Eq, PartialEq, Copy, Clone,
)]
pub enum BorderSide {
    #[default]
    Left,
    Right,
    Top,
    Bottom,
}
