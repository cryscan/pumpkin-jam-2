use bevy::{
    pbr::PbrPlugin,
    prelude::*,
    reflect::TypeUuid,
    render::{
        camera::{CameraRenderGraph, RenderTarget},
        render_resource::*,
        texture::ImageSampler,
        view::RenderLayers,
    },
    sprite::MaterialMesh2dBundle,
};
use bevy_hikari::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_wanderlust::{CharacterControllerBundle, ControllerInput, WanderlustPlugin};
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;
use std::f32::consts::PI;

/// This controls the resolution.
const RENDER_SIZE: [u32; 2] = [320, 180];
const RENDER_PASS_LAYER: RenderLayers = RenderLayers::layer(1);
const RENDER_IMAGE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Image::TYPE_UUID, 1145141919810);

const GROUND_SIZE: f32 = 100.0;
const CENTER_PILLAR_SIZE: f32 = 20.0;
const CUBE_SIZE: f32 = 1.0;

const LIGHT_ROTATION_SPEED: f32 = 0.1;

fn main() {
    App::new()
        .register_type::<Player>()
        .register_type::<PlayerCamera>()
        .register_type::<PlayerCatcher>()
        .register_type::<CatchObject>()
        .insert_resource(WindowDescriptor {
            width: 1280.,
            height: 720.,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgba(0.1, 0.1, 0.1, 1.0)))
        .insert_resource(HikariConfig {
            validation_interval: 1,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(WanderlustPlugin)
        .add_plugin(PbrPlugin)
        .add_plugin(HikariPlugin)
        .add_startup_system(setup_render.exclusive_system())
        .add_startup_system(lock_release_cursor)
        .add_startup_system(setup_scene)
        .add_system(toggle_release_cursor)
        .add_system(player_move)
        .add_system(player_look)
        .add_system(player_catch)
        .add_system(light_rotate_system)
        .run();
}

fn lock_release_cursor(mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        window.set_cursor_lock_mode(true);
        window.set_cursor_visibility(false);
    }
}

fn toggle_release_cursor(mut windows: ResMut<Windows>, keys: Res<Input<KeyCode>>) {
    if let Some(window) = windows.get_primary_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            window.set_cursor_lock_mode(!window.cursor_locked());
            window.set_cursor_visibility(!window.cursor_visible());
        }
    }
}

fn setup_render(
    mut commands: Commands,
    windows: Res<Windows>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: RENDER_SIZE[0],
        height: RENDER_SIZE[1],
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        sampler_descriptor: ImageSampler::Descriptor(SamplerDescriptor {
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..default()
        }),
        ..default()
    };

    image.resize(size);
    let image_handle = images.set(RENDER_IMAGE_HANDLE, image);

    let window = windows.primary();
    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        window.width(),
        window.height(),
    ))));

    let material_handle = materials.add(ColorMaterial {
        texture: Some(image_handle),
        ..default()
    });

    commands.spawn_bundle(MaterialMesh2dBundle {
        material: material_handle,
        mesh: quad_handle.into(),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 1.5),
            ..default()
        },
        ..default()
    });

    commands.spawn_bundle(Camera2dBundle {
        camera: Camera {
            priority: 0,
            ..default()
        },
        ..Camera2dBundle::default()
    });
}

#[derive(Debug, Actionlike, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Action {
    Move,
    Look,
    Jump,
    Catch,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Player {
    pub sensitivity: Vec2,
    pub speed: f32,
    pub max_catch_speed: f32,
    pub throw_speed: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            sensitivity: Vec2::new(0.1, 0.1),
            speed: 1.0,
            max_catch_speed: 100.0,
            throw_speed: 200.0,
        }
    }
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct PlayerCamera;

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct PlayerCatcher;

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct CatchObject;

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct EmissiveObject {
    timer: Timer,
    emissive: f32,
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _asset_server: Res<AssetServer>,
) {
    // Plane
    commands
        .spawn_bundle(SpatialBundle::default())
        .insert(Collider::cuboid(0.5 * GROUND_SIZE, 1.0, 0.5 * GROUND_SIZE))
        .with_children(|parent| {
            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(shape::Plane { size: GROUND_SIZE }.into()),
                    material: materials.add(StandardMaterial {
                        base_color: Color::rgb(0.8, 0.7, 0.6),
                        perceptual_roughness: 0.9,
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.0, 1.0, 0.0),
                    ..default()
                })
                .insert(RENDER_PASS_LAYER);
        });

    // Top
    commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_xyz(0.0, GROUND_SIZE * 0.5, 0.0),
            ..default()
        })
        .insert(Collider::cuboid(0.5 * GROUND_SIZE, 1.0, 0.5 * GROUND_SIZE));

    // Right
    commands
        .spawn_bundle(SpatialBundle {
            transform: Transform {
                translation: Vec3::new(0.5 * GROUND_SIZE, 0.0, 0.0),
                rotation: Quat::from_rotation_z(PI / 2.0),
                ..default()
            },
            ..default()
        })
        .insert(Collider::cuboid(0.5 * GROUND_SIZE, 1.0, 0.5 * GROUND_SIZE))
        .with_children(|parent| {
            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(shape::Plane { size: GROUND_SIZE }.into()),
                    material: materials.add(StandardMaterial {
                        base_color: Color::rgb(0.8, 0.7, 0.6),
                        perceptual_roughness: 0.9,
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.0, 1.0, 0.0),
                    ..default()
                })
                .insert(RENDER_PASS_LAYER);
        });

    // Left
    commands
        .spawn_bundle(SpatialBundle {
            transform: Transform {
                translation: Vec3::new(-0.5 * GROUND_SIZE, 0.0, 0.0),
                rotation: Quat::from_rotation_z(-PI / 2.0),
                ..default()
            },
            ..default()
        })
        .insert(Collider::cuboid(0.5 * GROUND_SIZE, 1.0, 0.5 * GROUND_SIZE))
        .with_children(|parent| {
            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(shape::Plane { size: GROUND_SIZE }.into()),
                    material: materials.add(StandardMaterial {
                        base_color: Color::rgb(0.8, 0.7, 0.6),
                        perceptual_roughness: 0.9,
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.0, 1.0, 0.0),
                    ..default()
                })
                .insert(RENDER_PASS_LAYER);
        });

    // Forward
    commands
        .spawn_bundle(SpatialBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, -0.5 * GROUND_SIZE),
                rotation: Quat::from_rotation_x(PI / 2.0),
                ..default()
            },
            ..default()
        })
        .insert(Collider::cuboid(0.5 * GROUND_SIZE, 1.0, 0.5 * GROUND_SIZE))
        .with_children(|parent| {
            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(shape::Plane { size: GROUND_SIZE }.into()),
                    material: materials.add(StandardMaterial {
                        base_color: Color::rgb(0.8, 0.7, 0.6),
                        perceptual_roughness: 0.9,
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.0, 1.0, 0.0),
                    ..default()
                })
                .insert(RENDER_PASS_LAYER);
        });

    // Backward
    commands
        .spawn_bundle(SpatialBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.5 * GROUND_SIZE),
                rotation: Quat::from_rotation_x(-PI / 2.0),
                ..default()
            },
            ..default()
        })
        .insert(Collider::cuboid(0.5 * GROUND_SIZE, 1.0, 0.5 * GROUND_SIZE))
        .with_children(|parent| {
            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(shape::Plane { size: GROUND_SIZE }.into()),
                    material: materials.add(StandardMaterial {
                        base_color: Color::rgb(0.8, 0.7, 0.6),
                        perceptual_roughness: 0.9,
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.0, 1.0, 0.0),
                    ..default()
                })
                .insert(RENDER_PASS_LAYER);
        });

    // Center
    commands
        .spawn_bundle(SpatialBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.25 * GROUND_SIZE, 0.0),
                ..default()
            },
            ..default()
        })
        .insert(Collider::cuboid(
            0.5 * CENTER_PILLAR_SIZE,
            0.5 * GROUND_SIZE,
            0.5 * CENTER_PILLAR_SIZE,
        ))
        .with_children(|parent| {
            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(
                        shape::Box::new(CENTER_PILLAR_SIZE, 0.5 * GROUND_SIZE, CENTER_PILLAR_SIZE)
                            .into(),
                    ),
                    material: materials.add(StandardMaterial {
                        base_color: Color::rgb(0.8, 0.7, 0.6),
                        perceptual_roughness: 0.9,
                        ..default()
                    }),
                    ..default()
                })
                .insert(RENDER_PASS_LAYER);
        });

    // Cubes
    let cube_mesh = meshes.add(shape::Cube::new(CUBE_SIZE).into());
    for id in 0..10 {
        commands
            .spawn_bundle(PbrBundle {
                mesh: cube_mesh.clone(),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.6, 0.7, 0.8),
                    emissive: Color::rgba(0.8, 0.7, 0.6, 0.1),
                    perceptual_roughness: 0.9,
                    ..default()
                }),
                transform: Transform::from_xyz(0.0, 2.0 + CUBE_SIZE * id as f32, 15.0),
                ..default()
            })
            .insert_bundle((
                RigidBody::Dynamic,
                Collider::cuboid(CUBE_SIZE * 0.5, CUBE_SIZE * 0.5, CUBE_SIZE * 0.5),
                ReadMassProperties::default(),
                Velocity::default(),
                ExternalImpulse::default(),
                Ccd::enabled(),
                CatchObject,
            ))
            .insert(RENDER_PASS_LAYER);
    }

    // Sphere
    // commands
    //     .spawn_bundle(PbrBundle {
    //         mesh: meshes.add(
    //             shape::Icosphere {
    //                 radius: BALL_RADIUS,
    //                 subdivisions: 3,
    //             }
    //             .into(),
    //         ),
    //         material: materials.add(StandardMaterial {
    //             base_color_texture: Some(asset_server.load("earth_daymap.jpg")),
    //             emissive: Color::rgba(1.0, 1.0, 1.0, 1.0),
    //             emissive_texture: Some(asset_server.load("earth_daymap.jpg")),
    //             perceptual_roughness: 0.9,
    //             ..default()
    //         }),
    //         transform: Transform::from_xyz(0.0, 2.0, -2.0),
    //         ..default()
    //     })
    //     .insert_bundle((
    //         RigidBody::Dynamic,
    //         Collider::ball(BALL_RADIUS),
    //         ReadMassProperties::default(),
    //         Velocity::default(),
    //         ExternalImpulse::default(),
    //         Ccd::enabled(),
    //         CatchObject,
    //     ))
    //     .insert(RENDER_PASS_LAYER);

    // Only directional light is supported
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 5.0, 0.0),
            rotation: Quat::from_euler(EulerRot::XYZ, -PI / 6.0, 0.0, 0.0),
            ..default()
        },
        ..default()
    });

    // Player
    commands
        .spawn_bundle(CharacterControllerBundle {
            transform: Transform::from_xyz(0.0, 2.0, 20.0),
            ..default()
        })
        .insert_bundle(InputManagerBundle::<Action> {
            input_map: InputMap::default()
                .insert(VirtualDPad::wasd(), Action::Move)
                .insert(DualAxis::left_stick(), Action::Move)
                .insert(DualAxis::mouse_motion(), Action::Look)
                .insert(DualAxis::right_stick(), Action::Look)
                .insert(KeyCode::Space, Action::Jump)
                .insert(MouseButton::Right, Action::Catch)
                .build(),
            ..default()
        })
        .insert(Player::default())
        .with_children(|parent| {
            // Camera
            parent
                .spawn_bundle(Camera3dBundle {
                    camera: Camera {
                        priority: -1,
                        target: RenderTarget::Image(RENDER_IMAGE_HANDLE.typed()),
                        ..default()
                    },
                    camera_render_graph: CameraRenderGraph::new(bevy_hikari::graph::NAME),
                    ..default()
                })
                .insert(RENDER_PASS_LAYER)
                .insert(PlayerCamera)
                .with_children(|parent| {
                    parent
                        .spawn_bundle(TransformBundle {
                            local: Transform::from_xyz(1.0, 1.0, -2.0),
                            ..default()
                        })
                        .insert(PlayerCatcher);
                });
        });
}

fn player_move(
    mut player: Query<(&ActionState<Action>, &Player, &mut ControllerInput)>,
    camera: Query<&GlobalTransform, (With<PlayerCamera>, Without<Player>)>,
) {
    let (action_state, player, mut controller) = player.single_mut();
    let camera = camera.single();

    let mut direction = Vec3::ZERO;
    if action_state.pressed(Action::Move) {
        let axis = action_state
            .clamped_axis_pair(Action::Move)
            .map_or(Vec2::ZERO, |axis| Vec2::new(axis.x(), axis.y()));
        direction = camera.right() * axis.x + camera.forward() * axis.y;
    }
    controller.movement = player.speed * direction.normalize_or_zero();
    controller.jumping = action_state.pressed(Action::Jump);
}

fn player_look(
    mut camera: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
    mut player: Query<(&ActionState<Action>, &Player, &mut Transform)>,
) {
    let mut camera = camera.single_mut();
    let (action_state, player, mut body) = player.single_mut();

    let mut delta = Vec2::ZERO;
    if action_state.pressed(Action::Look) {
        delta = action_state
            .axis_pair(Action::Look)
            .map_or(Vec2::ZERO, |axis| -Vec2::new(axis.x(), axis.y()));
    }

    camera.rotate_x(player.sensitivity.y * delta.y.to_radians());
    body.rotate_y(player.sensitivity.x * delta.x.to_radians());
}

fn player_catch(
    mut queries: ParamSet<(
        Query<(&ActionState<Action>, &Player)>,
        Query<&GlobalTransform, With<PlayerCatcher>>,
        Query<
            (
                &mut ExternalImpulse,
                &Velocity,
                &ReadMassProperties,
                &GlobalTransform,
            ),
            With<CatchObject>,
        >,
    )>,
) {
    let player_query = queries.p0();
    let (action_state, player) = player_query.single();

    let catch_pressed = action_state.pressed(Action::Catch);
    let catch_just_released = action_state.just_released(Action::Catch);

    let max_catch_speed = player.max_catch_speed;
    let throw_speed = player.throw_speed;

    let catcher_query = queries.p1();
    let catcher_transform = catcher_query.single();
    let catcher_position = catcher_transform.translation();
    let catcher_direction = catcher_transform.forward();

    // Find the closest catch object
    if let Some((mut impulse, velocity, mass, transform)) =
        queries.p2().iter_mut().min_by_key(|(_, _, _, transform)| {
            transform.translation().distance_squared(catcher_position) as u32
        })
    {
        let delta_position = catcher_position - transform.translation();
        if catch_pressed {
            let speed = (10.0 * delta_position.length_squared()).min(max_catch_speed);
            let delta_velocity = delta_position.normalize_or_zero() * speed - velocity.linvel;
            impulse.impulse = delta_velocity * mass.0.mass;
        } else if catch_just_released {
            let speed = 1.0 / (delta_position.length_squared() + 1.0) * throw_speed;
            let delta_velocity = catcher_direction * speed;
            impulse.impulse = delta_velocity * mass.0.mass;
        }
    }
}

fn light_rotate_system(time: Res<Time>, mut query: Query<&mut Transform, With<DirectionalLight>>) {
    for mut transform in &mut query {
        transform.rotate_y(LIGHT_ROTATION_SPEED * time.delta_seconds());
    }
}
