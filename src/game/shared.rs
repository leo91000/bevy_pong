use crate::game::protocol::Action;
use avian2d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;
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

pub fn apply_paddle_action(
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

    if direction != 0. {
        let new_y = transform.translation.y + direction * time.delta_secs() * 300.;
        transform.translation.y = new_y;
    }
}
