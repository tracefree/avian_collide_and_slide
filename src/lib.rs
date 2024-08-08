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
    let mut direction_result = Dir3::new(motion);
    let mut distance = motion.length();
    let original_velocity = motion;

    let Ok(start_direction) = direction_result else {
        return CollideAndSlideResult {
            slide_target: Vec3::ZERO,
            obstacle_hit: false,
        };
    };

    let mut bounce_count = 0;
    let mut hit_count = 0;

    let mut num_planes = 0;
    let mut planes = vec![Vec3::ZERO; config.max_clip_planes];

    let mut slide_target = Vec3::ZERO;

    'bounce_loop: for _ in 0..config.max_clip_planes {
        bounce_count += 1;

        if let Ok(direction) = direction_result {
            if let Some(hit) = spatial_query.cast_shape(
                collider,
                origin,
                Quat::IDENTITY, // TODO: Support rotation?
                direction,
                distance + config.skin_width,
                true,
                filter.clone(),
            ) {
                hit_count += 1;

                if hit.time_of_impact >= distance {
                    slide_target += direction * (hit.time_of_impact - config.skin_width).max(0.0);
                    break;
                }

                slide_target += direction * (hit.time_of_impact - config.skin_width);

                // If we move above a threshold, consider previous planes as no longer active obstacles.
                if (hit.time_of_impact - config.skin_width).abs() > 0.01 {
                    num_planes = 0;
                }

                // Too many obstacles, let's give up.
                if num_planes >= config.max_clip_planes {
                    break;
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
                    projected_velocity = original_velocity.reject_from_normalized(planes[i]);
                    for j in 0..num_planes {
                        if j != i && projected_velocity.dot(planes[j]) < 0.0 {
                            walk_along_crease = true;
                            break 'clip_planes;
                        }
                    }
                }

                if walk_along_crease {
                    if num_planes != 2 {
                        // Not sure about this...
                        break 'bounce_loop;
                    }
                    let crease_direction = planes[0].cross(planes[1]);
                    projected_velocity =
                        crease_direction * crease_direction.dot(projected_velocity);
                }

                // Avoid moving backwards
                if projected_velocity.dot(*start_direction) <= 0.0 {
                    break;
                }

                direction_result = Dir3::new(projected_velocity);
                distance = projected_velocity.length();
            } else {
                slide_target += direction * distance;
                break;
            }
        } else {
            break;
        }
    }

    return CollideAndSlideResult {
        slide_target,
        obstacle_hit: true,
    };
}

pub fn is_on_ground(
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
        0.1,
        true,
        filter.clone(),
    ) {
        hit.normal1.angle_between(Vec3::Y) < max_slide_angle
    } else {
        false
    }
}
