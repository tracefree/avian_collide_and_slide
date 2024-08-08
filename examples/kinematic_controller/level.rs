use std::f32::consts::PI;

use avian3d::prelude::*;
use bevy::{core_pipeline::experimental::taa::TemporalAntiAliasPlugin, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
    app.add_plugins(TemporalAntiAliasPlugin);
    app.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: light_consts::lux::DARK_OVERCAST_DAY,
    });
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

fn spawn_level(
    _trigger: Trigger<SpawnLevel>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Spawn geometry and construct colliders
    commands
        .spawn(SceneBundle {
            scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/gym.glb")),
            ..default()
        })
        .insert(ColliderConstructorHierarchy::new(
            ColliderConstructor::TrimeshFromMesh,
        ));

    // Spawn Light source
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::YXZ,
            PI / 4.0,
            -PI / 4.0,
            0.0,
        )),
        ..default()
    });
}
