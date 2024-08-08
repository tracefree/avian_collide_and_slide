use avian3d::{prelude::PhysicsDebugPlugin, PhysicsPlugins};
use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow, WindowResolution},
};
use level::SpawnLevel;
use player::SpawnPlayer;

mod controller;
mod input;
mod level;
mod player;

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum AppSet {
    RecordInput,
    Update,
}

fn main() {
    let mut app = App::new();

    app.configure_sets(Update, (AppSet::RecordInput, AppSet::Update).chain());

    app.add_plugins(
        DefaultPlugins.set(WindowPlugin {
            primary_window: Window {
                title: "Kinematic Character Controller".to_string(),
                resolution: WindowResolution::new(1920.0, 1080.0),
                ..default()
            }
            .into(),
            ..default()
        }),
    );

    app.add_plugins(PhysicsPlugins::default());
    app.add_plugins(PhysicsDebugPlugin::default());

    app.add_plugins((
        level::plugin,
        player::plugin,
        input::plugin,
        controller::plugin,
    ));

    app.add_systems(Startup, setup);

    app.run();
}

fn setup(mut commands: Commands, mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    commands.trigger(SpawnLevel);
    commands.trigger(SpawnPlayer);

    // Grab cursor
    let mut primary_window = windows.single_mut();
    primary_window.cursor.grab_mode = CursorGrabMode::Locked;
    primary_window.cursor.visible = false;
}
