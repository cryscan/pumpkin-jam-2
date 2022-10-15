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
use bevy_hikari::{prelude::*, HikariConfig};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_wanderlust::{CharacterControllerBundle, ControllerInput, WanderlustPlugin};
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;
use std::f32::consts::PI;

const RENDER_SIZE: [u32; 2] = [320, 180];
const RENDER_PASS_LAYER: RenderLayers = RenderLayers::layer(1);
const RENDER_IMAGE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Image::TYPE_UUID, 1145141919810);

const GROUND_SIZE: f32 = 100.0;
const _BALL_RADIUS: f32 = 0.5;
const CUBE_SIZE: f32 = 0.5;

fn main() {
    App::new()
        .register_type::<Player>()
        .register_type::<PlayerCamera>()
        .insert_resource(WindowDescriptor {
            width: 1280.,
            height: 720.,
            ..Default::default()
        })
        .insert_resource(HikariConfig {
            validation_interval: 1,
            emissive_threshold: 0.01,
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
        .add_system(bevy::window::close_on_esc)
        .add_system(player_move)
        .add_system(player_look)
        .run();
}

fn lock_release_cursor(mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        window.set_cursor_lock_mode(true);
        window.set_cursor_visibility(false);
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
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Player {
    pub speed: f32,
    pub sensitivity: Vec2,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: 1.0,
            sensitivity: Vec2::new(0.1, 0.1),
        }
    }
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct PlayerCamera;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _asset_server: Res<AssetServer>,
) {
    // Plane
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(shape::Plane { size: GROUND_SIZE }.into()),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.8, 0.7, 0.6),
                perceptual_roughness: 0.9,
                ..default()
            }),
            ..default()
        })
        .insert(Collider::cuboid(GROUND_SIZE * 0.5, 0.01, GROUND_SIZE * 0.5))
        .insert(RENDER_PASS_LAYER);

    // Cubes
    for id in -3..=3 {
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(shape::Cube::new(CUBE_SIZE).into()),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.6, 0.7, 0.8),
                    perceptual_roughness: 0.9,
                    ..default()
                }),
                transform: Transform::from_xyz(CUBE_SIZE * id as f32, 1.0, -6.0),
                ..default()
            })
            .insert_bundle((
                RigidBody::Dynamic,
                Collider::cuboid(CUBE_SIZE * 0.5, CUBE_SIZE * 0.5, CUBE_SIZE * 0.5),
            ))
            .insert(RENDER_PASS_LAYER);
    }

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(shape::Cube::new(CUBE_SIZE).into()),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.6, 0.7, 0.8),
                emissive: Color::rgba(0.8, 0.7, 0.6, 0.5),
                perceptual_roughness: 0.9,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 1.0, -2.0),
            ..default()
        })
        .insert_bundle((
            RigidBody::Dynamic,
            Collider::cuboid(CUBE_SIZE * 0.5, CUBE_SIZE * 0.5, CUBE_SIZE * 0.5),
        ))
        .insert(RENDER_PASS_LAYER);

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
    //             base_color: Color::rgb(0.6, 0.7, 0.8),
    //             emissive: Color::rgba(0.6, 0.7, 0.8, 0.5),
    //             perceptual_roughness: 0.9,
    //             ..default()
    //         }),
    //         transform: Transform::from_xyz(10.0, 1.0, -2.0),
    //         ..default()
    //     })
    //     .insert_bundle((RigidBody::Dynamic, Collider::ball(BALL_RADIUS)))
    //     .insert(RENDER_PASS_LAYER);

    // Only directional light is supported
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 5.0, 0.0),
            rotation: Quat::from_euler(EulerRot::XYZ, -PI / 4.0, PI / 4.0, 0.0),
            ..default()
        },
        ..default()
    });

    // Player
    commands
        .spawn_bundle(CharacterControllerBundle::default())
        // .insert_bundle(PbrBundle {
        //     mesh: meshes.add(shape::Capsule::default().into()),
        //     material: materials.add(StandardMaterial {
        //         base_color: Color::rgb(0.6, 0.7, 0.8),
        //         perceptual_roughness: 0.9,
        //         ..default()
        //     }),
        //     ..default()
        // })
        .insert_bundle(InputManagerBundle::<Action> {
            input_map: InputMap::default()
                .insert(VirtualDPad::wasd(), Action::Move)
                .insert(DualAxis::left_stick(), Action::Move)
                .insert(DualAxis::mouse_motion(), Action::Look)
                .insert(DualAxis::right_stick(), Action::Look)
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
                    // transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
                    ..default()
                })
                .insert(RENDER_PASS_LAYER)
                .insert(PlayerCamera);
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
