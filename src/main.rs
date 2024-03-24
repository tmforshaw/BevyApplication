//! This example demonstrates how to use the `Camera::viewport_to_world` method.

use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy_flycam::prelude::*;

const INITIAL_FOV: f32 = 75f32;

use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    prelude::*,
};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NoCameraPlayerPlugin)
        .insert_resource(MovementSettings {
            sensitivity: 0.00015, // default: 0.00012
            speed: 12.0,          // default: 12.0
        })
        .insert_resource(KeyBindings {
            move_ascend: KeyCode::KeyE,
            move_descend: KeyCode::KeyQ,
            ..Default::default()
        })
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (update_bloom_settings, bounce_spheres))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true, // 1. HDR is required for bloom
                ..default()
            },
            projection: PerspectiveProjection {
                fov: INITIAL_FOV.to_radians(),
                ..default()
            }
            .into(),
            tonemapping: Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        // 3. Enable bloom for the camera
        BloomSettings::NATURAL,
        FlyCam,
    ));

    let material_emissive1 = materials.add(StandardMaterial {
        emissive: Color::rgb_linear(23000.0, 9000.0, 3000.0), // 4. Put something bright in a dark environment to see the effect
        ..default()
    });
    let material_emissive2 = materials.add(StandardMaterial {
        emissive: Color::rgb_linear(3000.0, 23000.0, 9000.0),
        ..default()
    });
    let material_emissive3 = materials.add(StandardMaterial {
        emissive: Color::rgb_linear(9000.0, 3000.0, 23000.0),
        ..default()
    });
    let material_non_emissive = materials.add(StandardMaterial {
        base_color: Color::GRAY,
        ..default()
    });

    let mesh = meshes.add(Sphere::new(0.5).mesh().ico(5).unwrap());

    for x in -5..5 {
        for z in -5..5 {
            // This generates a pseudo-random integer between `[0, 6)`, but deterministically so
            // the same spheres are always the same colors.
            let mut hasher = DefaultHasher::new();
            (x, z).hash(&mut hasher);
            let rand = (hasher.finish() - 2) % 6;

            let material = match rand {
                0 => material_emissive1.clone(),
                1 => material_emissive2.clone(),
                2 => material_emissive3.clone(),
                3..=5 => material_non_emissive.clone(),
                _ => unreachable!(),
            };

            commands.spawn((
                PbrBundle {
                    mesh: mesh.clone(),
                    material,
                    transform: Transform::from_xyz(x as f32 * 2.0, 0.0, z as f32 * 2.0),
                    ..default()
                },
                Bouncing,
            ));
        }
    }

    // example instructions
    commands.spawn(
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
    );
}

// ------------------------------------------------------------------------------------------------

fn update_bloom_settings(
    mut camera: Query<(Entity, Option<&mut BloomSettings>), With<Camera>>,
    mut text: Query<&mut Text>,
    mut commands: Commands,
    keycode: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut key_evr: EventReader<KeyboardInput>,
    mut proj_query: Query<&mut Projection, With<FlyCam>>,
) {
    use bevy::input::ButtonState;

    let bloom_settings = camera.single_mut();
    let mut text = text.single_mut();
    let text = &mut text.sections[0].value;

    // assume perspective. do nothing if orthographic.
    let Projection::Perspective(persp) = proj_query.single_mut().into_inner() else {
        return;
    };

    match bloom_settings {
        (entity, Some(mut bloom_settings)) => {
            *text = "BloomSettings (Toggle: Space)\n".to_string();
            text.push_str(&format!("(P/;) Intensity: {}\n", bloom_settings.intensity));
            text.push_str(&format!(
                "(O/L) Low-frequency boost: {}\n",
                bloom_settings.low_frequency_boost
            ));
            text.push_str(&format!(
                "(I/K) Low-frequency boost curvature: {}\n",
                bloom_settings.low_frequency_boost_curvature
            ));
            text.push_str(&format!(
                "(U/J) High-pass frequency: {}\n",
                bloom_settings.high_pass_frequency
            ));
            text.push_str(&format!(
                "(Y/H) Mode: {}\n",
                match bloom_settings.composite_mode {
                    BloomCompositeMode::EnergyConserving => "Energy-conserving",
                    BloomCompositeMode::Additive => "Additive",
                }
            ));
            text.push_str(&format!(
                "(T/G) Threshold: {}\n",
                bloom_settings.prefilter_settings.threshold
            ));
            text.push_str(&format!(
                "(R/F) Threshold softness: {}\n",
                bloom_settings.prefilter_settings.threshold_softness
            ));
            text.push_str(&format!("([/]) FOV: {}\n", persp.fov.to_degrees()));

            let increase = 2f32.to_radians();

            let dt = time.delta_seconds();

            for ev in key_evr.read() {
                match ev.state {
                    ButtonState::Pressed => {
                        match ev.key_code {
                            KeyCode::BracketLeft | KeyCode::BracketRight => {
                                persp.fov += increase
                                    * if ev.key_code == KeyCode::BracketLeft {
                                        1f32
                                    } else {
                                        -1f32
                                    };

                                persp.fov = persp.fov.clamp(0f32, 180f32.to_radians())
                            }
                            KeyCode::KeyP | KeyCode::Semicolon => {
                                bloom_settings.intensity += dt / 10f32
                                    * if ev.key_code == KeyCode::KeyP {
                                        1f32
                                    } else {
                                        -1f32
                                    };

                                bloom_settings.intensity = bloom_settings.intensity.clamp(0.0, 1.0);
                            }
                            KeyCode::KeyO | KeyCode::KeyL => {
                                bloom_settings.low_frequency_boost += dt / 10f32
                                    * if ev.key_code == KeyCode::KeyO {
                                        1f32
                                    } else {
                                        -1f32
                                    };
                            }
                            KeyCode::KeyI | KeyCode::KeyK => {
                                bloom_settings.low_frequency_boost_curvature += dt / 10f32
                                    * if ev.key_code == KeyCode::KeyI {
                                        1f32
                                    } else {
                                        -1f32
                                    };

                                bloom_settings.low_frequency_boost_curvature =
                                    bloom_settings.low_frequency_boost_curvature.clamp(0.0, 1.0);
                            }
                            KeyCode::KeyU | KeyCode::KeyJ => {
                                bloom_settings.high_pass_frequency += dt / 10f32
                                    * if ev.key_code == KeyCode::KeyU {
                                        1f32
                                    } else {
                                        -1f32
                                    };

                                bloom_settings.high_pass_frequency =
                                    bloom_settings.high_pass_frequency.clamp(0.0, 1.0);
                            }
                            KeyCode::KeyY | KeyCode::KeyH => {
                                bloom_settings.composite_mode = if ev.key_code == KeyCode::KeyY {
                                    BloomCompositeMode::Additive
                                } else {
                                    BloomCompositeMode::EnergyConserving
                                };
                            }
                            KeyCode::KeyT | KeyCode::KeyG => {
                                bloom_settings.prefilter_settings.threshold += dt / 10f32
                                    * if ev.key_code == KeyCode::KeyT {
                                        1f32
                                    } else {
                                        -1f32
                                    };

                                bloom_settings.prefilter_settings.threshold =
                                    bloom_settings.prefilter_settings.threshold.max(0.0);
                            }
                            KeyCode::KeyR | KeyCode::KeyF => {
                                bloom_settings.prefilter_settings.threshold_softness += dt / 10f32
                                    * if ev.key_code == KeyCode::KeyR {
                                        1f32
                                    } else {
                                        -1f32
                                    };

                                bloom_settings.prefilter_settings.threshold_softness =
                                    bloom_settings
                                        .prefilter_settings
                                        .threshold_softness
                                        .clamp(0.0, 1.0);
                            }
                            KeyCode::Space => {
                                commands.entity(entity).remove::<BloomSettings>();
                            }
                            _ => {}
                        };
                    }
                    ButtonState::Released => {
                        // println!("Key release: {:?} ({:?})", ev.key_code, ev.logical_key);
                    }
                }
            }
        }

        (entity, None) => {
            *text = "Bloom: Off (Toggle: Space)".to_string();

            if keycode.just_pressed(KeyCode::Space) {
                commands.entity(entity).insert(BloomSettings::NATURAL);
            }
        }
    }
}

#[derive(Component)]
struct Bouncing;

fn bounce_spheres(time: Res<Time>, mut query: Query<&mut Transform, With<Bouncing>>) {
    for mut transform in query.iter_mut() {
        transform.translation.y =
            (transform.translation.x + transform.translation.z + time.elapsed_seconds()).sin();
    }
}
