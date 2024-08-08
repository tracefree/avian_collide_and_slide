use bevy::{input::mouse::MouseMotion, prelude::*};

use crate::AppSet;

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(PlayerInput::default());
    app.insert_resource(AccumulatedMouseMotion::default());
    app.add_systems(
        Update,
        (record_keyboard_input, record_mouse_motion).in_set(AppSet::RecordInput),
    );
    app.add_systems(PostUpdate, reset_mouse_motion);
}

#[derive(Resource, Default)]
pub struct PlayerInput {
    pub direction: Vec2,
    pub jump: bool,
    pub sprint: bool,
}

#[derive(Resource, Default)]
pub struct AccumulatedMouseMotion {
    pub delta: Vec2,
}

fn record_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_input: ResMut<PlayerInput>,
) {
    let mut direction = Vec2::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.0
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }

    direction = direction.normalize_or_zero();
    player_input.direction = direction;

    player_input.jump = keyboard_input.pressed(KeyCode::Space);
    player_input.sprint = keyboard_input.pressed(KeyCode::ShiftLeft);
}

fn record_mouse_motion(
    mut mouse_motion: EventReader<MouseMotion>,
    mut accumulated_mouse_motion: ResMut<AccumulatedMouseMotion>,
) {
    for motion in mouse_motion.read() {
        accumulated_mouse_motion.delta += motion.delta;
    }
}

fn reset_mouse_motion(mut accumulated_mouse_motion: ResMut<AccumulatedMouseMotion>) {
    accumulated_mouse_motion.delta = Vec2::ZERO;
}
