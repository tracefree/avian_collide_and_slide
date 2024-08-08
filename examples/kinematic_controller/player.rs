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
    println!("Spawning player");
    commands
        .spawn(MaterialMeshBundle {
            material: materials.add(StandardMaterial::from_color(Color::WHITE)),
            mesh: meshes.add(Capsule3d::new(0.3, 1.0)),
            transform: Transform::from_xyz(0.0, 0.8, 0.0),
            ..default()
        })
        .insert(Name::new("Player"))
        .insert(KinematicCharacterController::default())
        .insert(RigidBody::Kinematic)
        .insert(Collider::capsule(0.3, 1.0))
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
