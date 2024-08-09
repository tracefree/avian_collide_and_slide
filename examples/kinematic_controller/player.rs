use avian3d::prelude::*;
use bevy::{
    core_pipeline::experimental::taa::TemporalAntiAliasBundle,
    pbr::ScreenSpaceAmbientOcclusionBundle, prelude::*,
};

use crate::controller::KinematicCharacterController;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_player);
    app.insert_resource(Msaa::Off);
}

#[derive(Event, Debug)]
pub struct SpawnPlayer;

#[derive(Component)]
pub struct CameraPivot;

fn spawn_player(
    _trigger: Trigger<SpawnPlayer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut material = StandardMaterial::from_color(Color::WHITE);
    material.alpha_mode = AlphaMode::Blend;
    material.base_color = Color::srgba(1.0, 1.0, 1.0, 0.9);

    commands
        .spawn(MaterialMeshBundle {
            material: materials.add(material),
            mesh: meshes.add(Capsule3d::new(0.3, 1.0)),
            transform: Transform::from_xyz(0.0, 0.9, 0.0),
            ..default()
        })
        .insert(Name::new("Player"))
        .insert(KinematicCharacterController::default())
        .insert(RigidBody::Kinematic)
        .insert(Collider::capsule(0.3 - 0.05, 1.0)) // Shrink by skin width
        .with_children(|player| {
            player
                .spawn(CameraPivot)
                .insert(SpatialBundle {
                    transform: Transform::from_xyz(0.0, 0.4, 0.0),
                    ..default()
                })
                .with_children(|pivot| {
                    pivot
                        .spawn(Camera3dBundle {
                            transform: Transform::from_xyz(0.0, 0.0, 4.0),
                            ..default()
                        })
                        .insert(ScreenSpaceAmbientOcclusionBundle::default())
                        .insert(TemporalAntiAliasBundle::default());
                });
        });
}
