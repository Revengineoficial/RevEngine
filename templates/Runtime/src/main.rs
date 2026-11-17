use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::{gltf::GltfAssetLabel, prelude::*};
use bevy_rapier3d::prelude::*;
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Resource)]
struct GameProject {
    root: PathBuf,
}
#[derive(Deserialize, Default)]
struct SceneFile {
    #[serde(default)]
    models: Vec<ModelEntry>,
    #[serde(default)]
    objects: Vec<ObjectEntry>,
}
#[derive(Deserialize)]
struct ModelEntry {
    path: String,
    #[serde(default)]
    position: [f32; 3],
    #[serde(default = "one")]
    scale: f32,
}
#[derive(Deserialize)]
struct ObjectEntry {
    name: String,
    #[serde(default = "cube")]
    kind: String,
    #[serde(default)]
    position: [f32; 3],
    #[serde(default = "one")]
    size: f32,
    #[serde(default)]
    color: Option<[f32; 4]>,
    #[serde(default)]
    physics: bool,
}
fn one() -> f32 {
    1.0
}
fn cube() -> String {
    "cube".into()
}

fn main() {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let root = if cwd
        .file_name()
        .is_some_and(|n| n.eq_ignore_ascii_case("Runtime"))
    {
        cwd.parent().unwrap_or(&cwd).to_path_buf()
    } else {
        cwd
    };
    App::new()
        .insert_resource(GameProject { root: root.clone() })
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: root.to_string_lossy().to_string(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "RevEngine Game".into(),
                        resolution: (1280.0_f32, 720.0_f32).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(AmbientLight {
            color: Color::srgb(0.72, 0.78, 0.9),
            brightness: 260.0,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                rotate_demo,
                player_controller,
                interact_nearby,
                exit_on_escape,
            ),
        )
        .run();
}
fn setup(
    mut commands: Commands,
    project: Res<GameProject>,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 8.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 14_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 9.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        PointLight {
            intensity: 900_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-4.0, 4.0, -2.0),
    ));
    let scene_path = project.root.join("Scenes").join("Main.scene.json");
    let scene = fs::read_to_string(scene_path)
        .ok()
        .and_then(|s| serde_json::from_str::<SceneFile>(&s).ok())
        .unwrap_or_default();
    let had_models = !scene.models.is_empty();
    let mut models = scene.models;
    if models.is_empty()
        && project
            .root
            .join("Assets/Generated/generated_scene.glb")
            .exists()
    {
        models.push(ModelEntry {
            path: "Assets/Generated/generated_scene.glb".into(),
            position: [0.0, 0.0, 0.0],
            scale: 1.0,
        });
    }
    for model in models {
        let path = model.path.replace('\\', "/");
        commands.spawn((
            SceneRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(path))),
            Transform::from_translation(Vec3::from_array(model.position))
                .with_scale(Vec3::splat(model.scale)),
        ));
    }
    if scene.objects.is_empty()
        && !had_models
        && !project
            .root
            .join("Assets/Generated/generated_scene.glb")
            .exists()
    {
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(30.0, 30.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.10, 0.13, 0.18))),
            Transform::from_xyz(0.0, -0.1, 0.0),
            RigidBody::Fixed,
            Collider::cuboid(15.0, 0.1, 15.0),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.8, 1.8, 1.8))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.12, 0.42, 0.88),
                metallic: 0.25,
                perceptual_roughness: 0.28,
                ..default()
            })),
            Transform::from_xyz(0.0, 1.0, 0.0),
            Rotates,
        ));
    }
    for object in scene.objects {
        let mesh = match object.kind.as_str() {
            "sphere" => Sphere::new(object.size * 0.5).mesh().uv(32, 18),
            "plane" => Plane3d::default()
                .mesh()
                .size(object.size, object.size)
                .build(),
            "capsule" => Capsule3d::new(object.size * 0.25, object.size * 0.5).into(),
            _ => Cuboid::new(object.size, object.size, object.size).into(),
        };
        let c = object.color.unwrap_or([0.24, 0.48, 0.82, 1.0]);
        let material = StandardMaterial {
            base_color: Color::srgba(c[0], c[1], c[2], c[3]),
            metallic: 0.12,
            perceptual_roughness: 0.42,
            ..default()
        };
        let mut entity = commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(material)),
            Transform::from_translation(Vec3::from_array(object.position)),
            Name::new(object.name),
        ));
        if object.kind == "plane" {
            entity.insert((
                RigidBody::Fixed,
                Collider::cuboid(object.size * 0.5, 0.05, object.size * 0.5),
            ));
        } else if object.physics {
            entity.insert((
                RigidBody::Dynamic,
                Collider::ball(object.size * 0.5),
                Restitution::coefficient(0.25),
            ));
        }
    }
}
#[derive(Component)]
struct Rotates;
fn rotate_demo(time: Res<Time>, mut q: Query<&mut Transform, With<Rotates>>) {
    for mut t in &mut q {
        t.rotate_y(time.delta_secs() * 0.55);
    }
}
// First-person 3D controller: WASD to walk, Space/Ctrl to fly vertically,
// hold right mouse button to look around, E to inspect/interact with nearby props.
fn player_controller(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };
    let dt = time.delta_secs().min(0.05);
    if mouse_buttons.pressed(MouseButton::Right) {
        let delta = mouse_motion.delta;
        transform.rotate_y(-delta.x * 0.0025);
        transform.rotate_local_x(-delta.y * 0.0025);
    }
    let mut direction = Vec3::ZERO;
    let forward = (transform.rotation * Vec3::NEG_Z).normalize_or_zero();
    let right = (transform.rotation * Vec3::X).normalize_or_zero();
    let forward_flat = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let right_flat = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();
    if keys.pressed(KeyCode::KeyW) {
        direction += forward_flat;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction -= forward_flat;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += right_flat;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction -= right_flat;
    }
    if keys.pressed(KeyCode::Space) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::ControlLeft) {
        direction.y -= 1.0;
    }
    if direction.length_squared() > 0.0 {
        transform.translation += direction.normalize() * 5.0 * dt;
    }
}

fn interact_nearby(
    keys: Res<ButtonInput<KeyCode>>,
    camera: Query<&Transform, With<Camera3d>>,
    targets: Query<(&Transform, &Name)>,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok(camera) = camera.single() else {
        return;
    };
    let nearest = targets
        .iter()
        .filter_map(|(transform, name)| {
            let distance = transform.translation.distance(camera.translation);
            (distance <= 3.0).then_some((distance, name.as_str()))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0));
    if let Some((distance, name)) = nearest {
        println!("[INTERAÇÃO] {} (distância {:.1}m)", name, distance);
    } else {
        println!("[INTERAÇÃO] Nenhum objeto nomeado a até 3 metros.");
    }
}

fn exit_on_escape(keys: Res<ButtonInput<KeyCode>>, mut app_exit: EventWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        app_exit.write(AppExit::Success);
    }
}
