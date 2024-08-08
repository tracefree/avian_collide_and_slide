use std::f32::consts::PI;

use avian3d::prelude::*;
use avian_collide_and_slide::{collide_and_slide, CollideAndSlideConfig};
use bevy::prelude::*;

use crate::{
    input::{AccumulatedMouseMotion, PlayerInput},
    player::CameraPivot,
    AppSet,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, apply_rotation.in_set(AppSet::Update));
    app.add_systems(
        FixedUpdate,
        (apply_movement, apply_damping).in_set(AppSet::Update),
    );
}

#[derive(Component)]
pub struct KinematicCharacterController {
    pub max_speed: f32,
    pub acceleration: f32,
    pub grounded: bool,
    pub velocity: Vec3,
    pub jump_impulse: f32,
    pub collide_and_slide_config: CollideAndSlideConfig,
}

impl Default for KinematicCharacterController {
    fn default() -> Self {
        Self {
            max_speed: 10.0,
            acceleration: 30.0,
            grounded: false,
            velocity: Vec3::ZERO,
            jump_impulse: 5.0,
            collide_and_slide_config: CollideAndSlideConfig {
                up_direction: Dir3::Y,
                max_slope_angle: PI / 4.0,
                floor_snap_length: 0.1,
                constant_speed: false,
                skin_width: 0.05,
                max_bounces: 4,
                max_clip_planes: 5,
            },
        }
    }
}

fn apply_rotation(
    mut q_character: Query<&mut Transform, With<KinematicCharacterController>>,
    mut q_camera_pivot: Query<
        &mut Transform,
        (With<CameraPivot>, Without<KinematicCharacterController>),
    >,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
) {
    // TODO: Don't assume there is only a single character
    let mut player = q_character.single_mut();
    let mut camera = q_camera_pivot.single_mut();
    let yaw = -accumulated_mouse_motion.delta.x * 0.003;
    let pitch = -accumulated_mouse_motion.delta.y * 0.002;
    let sensititvity = 0.5;
    player.rotate_y(sensititvity * yaw);
    camera.rotate_local_x(sensititvity * pitch);
}

fn apply_movement(
    mut q_controllers: Query<(
        Entity,
        &mut KinematicCharacterController,
        &mut Transform,
        &Rotation,
        &Collider,
    )>,
    input: Res<PlayerInput>,
    time: Res<Time>,
    spatial_query: SpatialQuery,
) {
    for (entity, mut controller, mut transform, rotation, collider) in q_controllers.iter_mut() {
        let motion = rotation.mul_vec3(Vec3::new(
            input.direction.x * controller.acceleration,
            0.0,
            -input.direction.y * controller.acceleration,
        )) * time.delta_seconds()
            * controller.acceleration;

        controller.velocity += motion;
        //controller.velocity.y -= 9.8 * time.delta_seconds();

        let sprint_modifier = if input.sprint { 1.5 } else { 1.0 };

        if controller.velocity.with_y(0.0).length() >= controller.max_speed * sprint_modifier {
            let clamped_horizontal_velocity = controller.velocity.with_y(0.0).normalize()
                * controller.max_speed
                * sprint_modifier;
            controller.velocity = clamped_horizontal_velocity.with_y(controller.velocity.y);
        }

        if input.jump && controller.grounded {
            controller.velocity.y = controller.jump_impulse;
        }

        let new_motion = collide_and_slide(
            transform.translation,
            controller.velocity * time.delta_seconds(),
            collider,
            &controller.collide_and_slide_config,
            &spatial_query,
            &SpatialQueryFilter::from_excluded_entities([entity]),
        );

        transform.translation += new_motion;
    }
}

fn apply_damping(mut q_controllers: Query<&mut KinematicCharacterController>) {
    for mut controller in &mut q_controllers {
        controller.velocity.x *= 0.9;
        controller.velocity.z *= 0.9;
    }
}
