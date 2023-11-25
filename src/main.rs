use bevy::{prelude::*, DefaultPlugins, pbr::wireframe::{ WireframeConfig, WireframePlugin},};
use bevy_flycam::prelude::*;
use world::WorldPlugin;
use bevy::window::PresentMode;
use bevy::pbr::CascadeShadowConfigBuilder;
use std::f32::consts::PI;
mod world;

#[bevy_main]
fn main() {
    App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            // Turn off vsync to maximize CPU/GPU usage
            present_mode: PresentMode::AutoNoVsync,
            ..default()
        }),
        ..default()
    }))
        .add_plugins(WireframePlugin)
        .insert_resource(WireframeConfig {
            global: true,
            default_color: Color::WHITE,
        })
        .add_plugins(NoCameraPlayerPlugin)
        .add_plugins(WorldPlugin)
        .add_systems(Startup,  spawn_light)
        .add_systems(Update, animate_light_direction)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(5.0, 120.0, 5.5),
            ..default()
        },
        FlyCam
    ));
}


fn spawn_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 10.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 500.0,
            ..default()
        }
        .into(),
        ..default()
    });
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() * 0.5);
    }
}