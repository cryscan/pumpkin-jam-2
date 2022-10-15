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
use bevy_rapier3d::prelude::*;
use std::f32::consts::PI;

const RENDER_SIZE: [u32; 2] = [320, 180];
const RENDER_PASS_LAYER: RenderLayers = RenderLayers::layer(1);
const RENDER_IMAGE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Image::TYPE_UUID, 1145141919810);

const GROUND_SIZE: f32 = 20.0;
const BALL_RADIUS: f32 = 0.5;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1280.,
            height: 720.,
            ..Default::default()
        })
        .insert_resource(HikariConfig {
            validation_interval: 1,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(PbrPlugin)
        .add_plugin(HikariPlugin)
        .add_startup_system(setup_render)
        .add_startup_system(setup_entities.after(setup_render))
        .run();
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

fn setup_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _asset_server: Res<AssetServer>,
) {
    // Plane
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: GROUND_SIZE })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.8, 0.7, 0.6),
                perceptual_roughness: 0.9,
                ..default()
            }),
            ..default()
        })
        .insert_bundle((
            Collider::cuboid(GROUND_SIZE, 0.1, GROUND_SIZE),
            RENDER_PASS_LAYER,
        ));

    // Sphere
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: BALL_RADIUS,
                subdivisions: 3,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.6, 0.7, 0.8),
                emissive: Color::rgba(0.6, 0.7, 0.8, 0.5),
                perceptual_roughness: 0.9,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        })
        .insert_bundle((
            RigidBody::Dynamic,
            Collider::ball(BALL_RADIUS),
            RENDER_PASS_LAYER,
        ));

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

    // Camera
    commands
        .spawn_bundle(Camera3dBundle {
            camera: Camera {
                priority: -1,
                target: RenderTarget::Image(RENDER_IMAGE_HANDLE.typed()),
                ..default()
            },
            camera_render_graph: CameraRenderGraph::new(bevy_hikari::graph::NAME),
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(RENDER_PASS_LAYER);
}
