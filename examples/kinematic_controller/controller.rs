use avian3d::prelude::*;
use avian_collide_and_slide::{
    collide_and_slide, is_on_floor, snap_to_floor, CollideAndSlideConfig,
};
use bevy::prelude::*;

use crate::{
    input::{AccumulatedMouseMotion, PlayerInput},
    player::CameraPivot,
    AppSet,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, apply_rotation.in_set(AppSet::Update));

    // Todo: What is the correct schedule and system ordering for this?
    app.add_systems(
        FixedPostUpdate,
        (
            update_grounded,
            apply_gravity,
            apply_movement,
            apply_damping,
        )
            .chain()
            .before(PhysicsSet::Sync)
            .before(TransformSystem::TransformPropagate), // .in_set(AppSet::Update),
    );
}

#[derive(Component)]
pub struct KinematicCharacterController {
    pub _max_speed: f32,
    pub acceleration: f32,
    pub grounded: bool,
    pub velocity: Vec3,
    pub jump_impulse: f32,
    pub collide_and_slide_config: CollideAndSlideConfig,
}

impl Default for KinematicCharacterController {
    fn default() -> Self {
        Self {
            _max_speed: 10.0,
            acceleration: 8.0,
            grounded: false,
            velocity: Vec3::ZERO,
            jump_impulse: 5.0,
            collide_and_slide_config: CollideAndSlideConfig {
                up_direction: Dir3::Y,
                max_slope_angle: 45.0_f32.to_radians(),
                floor_snap_length: 0.4,
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
        // Construct desired motion vector from player input
        // Horizontal motion
        let motion = rotation.mul_vec3(Vec3::new(
            input.direction.x * controller.acceleration,
            0.0,
            -input.direction.y * controller.acceleration,
        )) * time.delta_seconds()
            * controller.acceleration;
        controller.velocity += motion;

        // Jump
        if input.jump && controller.grounded {
            controller.velocity.y = controller.jump_impulse;
        }

        // Collision resolution. Split into horizontal and vertical motion passes to make it easier to overcome small obstacles.
        // Has no impact on performance, the two passes have fewer bounces than one combined pass would have.
        let filter = &SpatialQueryFilter::from_excluded_entities([entity]);

        // - Horizontal pass
        let mut new_motion = collide_and_slide(
            transform.translation,
            controller.velocity.with_y(0.0) * time.delta_seconds(),
            collider,
            &controller.collide_and_slide_config,
            &spatial_query,
            &filter,
        )
        .slide_target;

        // - Gravity pass
        new_motion += collide_and_slide(
            transform.translation + new_motion,
            controller.velocity.with_x(0.0).with_z(0.0) * time.delta_seconds(),
            collider,
            &controller.collide_and_slide_config,
            &spatial_query,
            &filter,
        )
        .slide_target;

        // Snap to floor
        if controller.grounded
            && controller.velocity.y <= 0.0
            && controller.collide_and_slide_config.floor_snap_length > 0.0
        {
            let snap_motion = snap_to_floor(
                transform.translation + new_motion,
                collider,
                controller.collide_and_slide_config.floor_snap_length,
                controller.collide_and_slide_config.max_slope_angle,
                &spatial_query,
                &filter,
            );

            new_motion += snap_motion;
        }

        // Apply new position
        transform.translation += new_motion;
    }
}

fn apply_damping(mut q_controllers: Query<&mut KinematicCharacterController>) {
    for mut controller in &mut q_controllers {
        controller.velocity.x *= 0.9;
        controller.velocity.z *= 0.9;
    }
}

fn update_grounded(
    mut q_controllers: Query<(
        Entity,
        &mut KinematicCharacterController,
        &Transform,
        &Collider,
    )>,
    spatial_query: SpatialQuery,
) {
    for (entity, mut controller, transform, collider) in &mut q_controllers {
        controller.grounded = is_on_floor(
            transform.translation,
            collider,
            controller.collide_and_slide_config.max_slope_angle,
            &spatial_query,
            &SpatialQueryFilter::from_excluded_entities([entity]),
        );
    }
}

fn apply_gravity(mut q_controllers: Query<&mut KinematicCharacterController>, time: Res<Time>) {
    for mut controller in &mut q_controllers {
        if !controller.grounded {
            controller.velocity.y -= 9.8 * time.delta_seconds();
        } else {
            controller.velocity.y = 0.0;
        }
    }
}
