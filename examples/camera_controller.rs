//! A minimal fly camera, so the examples do not depend on a third-party camera crate that has to
//! be upgraded in lockstep with Bevy.
//!
//! Hold the right mouse button to look around; `WASD` moves, `Q`/`E` go down/up and shift is a
//! speed boost.

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, run_camera_controller);
    }
}

#[derive(Component)]
pub struct CameraController {
    pub speed: f32,
    pub boost: f32,
    pub sensitivity: f32,
    yaw: f32,
    pitch: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            speed: 200.0,
            boost: 5.0,
            sensitivity: 0.002,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

impl CameraController {
    /// Builds a controller already aimed from `eye` at `target`, to match the transform the camera
    /// is spawned with.
    pub fn looking_at(eye: Vec3, target: Vec3) -> (Self, Transform) {
        let transform = Transform::from_translation(eye).looking_at(target, Vec3::Y);
        let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        (
            Self {
                yaw,
                pitch,
                ..default()
            },
            transform,
        )
    }
}

fn run_camera_controller(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera: Query<(&mut CameraController, &mut Transform)>,
) {
    let Ok((mut controller, mut transform)) = camera.single_mut() else {
        return;
    };

    // Only steer while the right mouse button is held, so the cursor stays usable otherwise.
    if mouse_buttons.pressed(MouseButton::Right) && !windows.is_empty() {
        let delta = mouse_motion.delta;
        controller.yaw -= delta.x * controller.sensitivity;
        controller.pitch = (controller.pitch - delta.y * controller.sensitivity).clamp(
            -core::f32::consts::FRAC_PI_2 + 0.01,
            core::f32::consts::FRAC_PI_2 - 0.01,
        );
        transform.rotation = Quat::from_euler(EulerRot::YXZ, controller.yaw, controller.pitch, 0.0);
    }

    let mut direction = Vec3::ZERO;
    for (key, axis) in [
        (KeyCode::KeyW, Vec3::NEG_Z),
        (KeyCode::KeyS, Vec3::Z),
        (KeyCode::KeyA, Vec3::NEG_X),
        (KeyCode::KeyD, Vec3::X),
        (KeyCode::KeyE, Vec3::Y),
        (KeyCode::KeyQ, Vec3::NEG_Y),
    ] {
        if keys.pressed(key) {
            direction += axis;
        }
    }

    if direction != Vec3::ZERO {
        let speed = if keys.pressed(KeyCode::ShiftLeft) {
            controller.speed * controller.boost
        } else {
            controller.speed
        };
        let translation = transform.rotation * direction.normalize() * speed * time.delta_secs();
        transform.translation += translation;
    }
}
