use bevy::{
    camera::Hdr,
    color::palettes::css,
    core_pipeline::tonemapping::Tonemapping,
    post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter},
    prelude::*,
};
use bevy_aabb_instancing::{Cuboid, CuboidMaterialId, Cuboids, VertexPullingRenderPlugin};

mod camera_controller;
use camera_controller::{CameraController, CameraControllerPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            VertexPullingRenderPlugin { outlines: true },
            CameraControllerPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let colors = [css::RED, css::GREEN, css::BLUE, css::YELLOW, css::PURPLE];

    let mut cuboids = Vec::new();
    for x in 0..10 {
        for y in 0..10 {
            let min = Vec3::new(x as f32 - 5.0, 0.0, y as f32 - 5.0);
            let max = min + Vec3::ONE;
            let color = u32::from_le_bytes(colors[(x + y) % colors.len()].to_u8_array());
            let mut cuboid = Cuboid::new(min, max, color);
            if min.length() < 3.0 {
                cuboid.make_emissive();
            }
            cuboids.push(cuboid);
        }
    }

    let cuboids = Cuboids::new(cuboids);
    let aabb = cuboids.aabb();
    commands.spawn((
        Transform::default(),
        Visibility::default(),
        cuboids,
        aabb,
        CuboidMaterialId(0),
    ));

    let (controller, transform) = CameraController::looking_at(Vec3::splat(10.0), Vec3::ZERO);
    commands.spawn((
        Camera3d::default(),
        // Bloom reads the HDR intermediate texture, and needs a non-multisampled target.
        Hdr,
        Msaa::Off,
        Tonemapping::TonyMcMapface,
        Bloom {
            intensity: 0.2,
            high_pass_frequency: 1.0,
            low_frequency_boost: 0.8,
            low_frequency_boost_curvature: 0.7,
            prefilter: BloomPrefilter {
                threshold: 0.0,
                threshold_softness: 0.0,
            },
            composite_mode: BloomCompositeMode::EnergyConserving,
            ..default()
        },
        controller,
        transform,
    ));
}
