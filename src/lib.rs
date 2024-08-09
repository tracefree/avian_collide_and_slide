use avian3d::prelude::*;
use bevy::prelude::*;

pub struct CollideAndSlideConfig {
    pub up_direction: Dir3,
    pub skin_width: f32,
    pub max_slope_angle: f32,
    pub floor_snap_length: f32,
    pub constant_speed: bool,
    pub max_bounces: usize,
    pub max_clip_planes: usize,
}

pub struct CollideAndSlideResult {
    pub slide_target: Vec3,
    pub obstacle_hit: bool,
}

pub fn collide_and_slide(
    origin: Vec3,
    motion: Vec3,
    collider: &Collider,
    config: &CollideAndSlideConfig,
    spatial_query: &SpatialQuery,
    filter: &SpatialQueryFilter,
) -> CollideAndSlideResult {
    let mut direction = if let Ok(start_direction) = Dir3::new(motion) {
        start_direction
    } else {
        return CollideAndSlideResult {
            slide_target: Vec3::ZERO,
            obstacle_hit: false,
        };
    };
    let mut distance = motion.length();

    let mut num_planes = 0;
    let mut planes = vec![Vec3::ZERO; config.max_clip_planes];

    // The new motion vector that we return in the end
    let mut slide_target = Vec3::ZERO;

    'bounce_loop: for _ in 0..config.max_bounces {
        if let Some(hit) = spatial_query.cast_shape(
            collider,
            origin,
            Quat::IDENTITY, // TODO: Support rotation?
            direction,
            distance + config.skin_width,
            true,
            filter.clone(),
        ) {
            if hit.time_of_impact >= distance {
                // Obstacle only appears in the skin width margin, move as far as allowed.
                // (TODO: This does not keep the body at a skin's distance when hitting obstacle at an angle)
                slide_target += direction * (hit.time_of_impact - config.skin_width).max(0.0);
                break 'bounce_loop;
            }

            // Move as close obstacle as we can.
            slide_target += direction * (hit.time_of_impact - config.skin_width);

            // If we moved above a threshold, consider previous planes as no longer active obstacles.
            if (hit.time_of_impact - config.skin_width).abs() > 0.01 {
                num_planes = 0;
            }

            // Too many obstacles, let's give up.
            if num_planes >= config.max_clip_planes {
                break 'bounce_loop;
            }

            // Add the obstacle we hit to the sliding planes.
            planes[num_planes] = hit.normal1;
            num_planes += 1;

            let extra_distance = distance - (hit.time_of_impact - config.skin_width).max(0.0);
            let extra_velocity = direction * extra_distance;

            // Inspired by Quake's collision resolution
            let mut projected_velocity = extra_velocity;
            let mut walk_along_crease = false;
            'clip_planes: for i in 0..num_planes {
                projected_velocity = motion.reject_from_normalized(planes[i]);
                for j in 0..num_planes {
                    if j != i && projected_velocity.dot(planes[j]) < 0.0 {
                        walk_along_crease = true;
                        break 'clip_planes;
                    }
                }
            }

            if walk_along_crease {
                if num_planes != 2 {
                    // TODO: If there are more than two planes, select best pair for the crease
                    break 'bounce_loop;
                }
                let crease_direction = planes[0].cross(planes[1]);
                projected_velocity = crease_direction * crease_direction.dot(projected_velocity);
            }

            // Avoid moving backwards
            if projected_velocity.dot(motion) <= 0.0 {
                break 'bounce_loop;
            }

            // Set up the next iteration of the bounce loop if the projected motion is normalizable
            direction = if let Ok(new_direction) = Dir3::new(projected_velocity) {
                new_direction
            } else {
                break 'bounce_loop;
            };
            distance = projected_velocity.length();
        } else {
            // No obstacles, move the entire distance
            slide_target += direction * distance;
            break 'bounce_loop;
        }
    }

    return CollideAndSlideResult {
        slide_target,
        obstacle_hit: true,
    };
}

pub fn snap_to_floor(
    origin: Vec3,
    collider: &Collider,
    floor_snap_length: f32,
    _max_slide_angle: f32,
    spatial_query: &SpatialQuery,
    filter: &SpatialQueryFilter,
) -> Vec3 {
    if let Some(hit) = spatial_query.cast_shape(
        collider,
        origin,
        Quat::IDENTITY, // TODO: Support rotation?
        Dir3::NEG_Y,
        floor_snap_length + 0.05, // TODO: Don't hardcode this value
        true,
        filter.clone(),
    ) {
        return (hit.time_of_impact - 0.05 * hit.normal2.angle_between(Vec3::NEG_Y).cos())
            * Vec3::NEG_Y;
    }

    Vec3::ZERO
}

pub fn is_on_floor(
    origin: Vec3,
    collider: &Collider,
    max_slide_angle: f32,
    spatial_query: &SpatialQuery,
    filter: &SpatialQueryFilter,
) -> bool {
    if let Some(hit) = spatial_query.cast_shape(
        collider,
        origin,
        Quat::IDENTITY, // TODO: Support rotation?
        Dir3::NEG_Y,
        0.2, // TODO: Don't hardcode this value
        true,
        filter.clone(),
    ) {
        hit.normal1.angle_between(Vec3::Y) < max_slide_angle
    } else {
        false
    }
}
