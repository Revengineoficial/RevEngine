use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transform {
    #[serde(default)]
    pub position: [f32; 3],
    #[serde(default)]
    pub rotation: [f32; 3], // pitch, yaw, roll in degrees
    #[serde(default = "default_scale")]
    pub scale: [f32; 3],
}

fn default_scale() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialData {
    #[serde(default = "default_color")]
    pub color: [f32; 4],
    #[serde(default = "default_metallic")]
    pub metallic: f32,
    #[serde(default = "default_roughness")]
    pub roughness: f32,
    #[serde(default)]
    pub emissive: [f32; 3],
    #[serde(default)]
    pub texture_path: Option<String>,
}

fn default_color() -> [f32; 4] {
    [0.75, 0.75, 0.78, 1.0]
}
fn default_metallic() -> f32 {
    0.1
}
fn default_roughness() -> f32 {
    0.45
}

impl Default for MaterialData {
    fn default() -> Self {
        Self {
            color: default_color(),
            metallic: default_metallic(),
            roughness: default_roughness(),
            emissive: [0.0, 0.0, 0.0],
            texture_path: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhysicsData {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_body_type")]
    pub body_type: String, // "Dynamic", "Static", "Kinematic"
    #[serde(default = "default_mass")]
    pub mass: f32,
    #[serde(default = "default_friction")]
    pub friction: f32,
    #[serde(default = "default_restitution")]
    pub restitution: f32,
    #[serde(default = "default_collider")]
    pub collider: String, // "box", "sphere", "capsule", "mesh"
}

fn default_body_type() -> String {
    "Dynamic".into()
}
fn default_mass() -> f32 {
    1.0
}
fn default_friction() -> f32 {
    0.5
}
fn default_restitution() -> f32 {
    0.1
}
fn default_collider() -> String {
    "box".into()
}

impl Default for PhysicsData {
    fn default() -> Self {
        Self {
            enabled: false,
            body_type: default_body_type(),
            mass: default_mass(),
            friction: default_friction(),
            restitution: default_restitution(),
            collider: default_collider(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LightData {
    pub light_type: String, // "directional", "point", "spot"
    #[serde(default = "default_light_color")]
    pub color: [f32; 3],
    #[serde(default = "default_light_intensity")]
    pub intensity: f32,
    #[serde(default = "default_light_range")]
    pub range: f32,
    #[serde(default = "default_true")]
    pub cast_shadows: bool,
}

fn default_light_color() -> [f32; 3] {
    [1.0, 0.95, 0.88]
}
fn default_light_intensity() -> f32 {
    1000.0
}
fn default_light_range() -> f32 {
    15.0
}
fn default_true() -> bool {
    true
}

impl Default for LightData {
    fn default() -> Self {
        Self {
            light_type: "point".into(),
            color: default_light_color(),
            intensity: default_light_intensity(),
            range: default_light_range(),
            cast_shadows: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CameraData {
    #[serde(default = "default_fov")]
    pub fov: f32,
    #[serde(default = "default_near")]
    pub near: f32,
    #[serde(default = "default_far")]
    pub far: f32,
    #[serde(default)]
    pub orthographic: bool,
}

fn default_fov() -> f32 {
    60.0
}
fn default_near() -> f32 {
    0.1
}
fn default_far() -> f32 {
    1000.0
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            fov: default_fov(),
            near: default_near(),
            far: default_far(),
            orthographic: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    pub id: u64,
    pub name: String,
    pub kind: String, // "cube", "sphere", "plane", "cylinder", "capsule", "cone", "torus", "monkey", "light", "camera", "model", "empty"
    #[serde(default)]
    pub transform: Transform,
    #[serde(default)]
    pub material: MaterialData,
    #[serde(default)]
    pub physics: PhysicsData,
    #[serde(default)]
    pub light: Option<LightData>,
    #[serde(default)]
    pub camera: Option<CameraData>,
    #[serde(default)]
    pub model_path: Option<String>,
    #[serde(default)]
    pub script_path: Option<String>,
    #[serde(default = "default_true")]
    pub visible: bool,
}

impl Entity {
    pub fn new_cube(id: u64, name: &str, pos: [f32; 3], size: f32, color: [f32; 4]) -> Self {
        Self {
            id,
            name: name.into(),
            kind: "cube".into(),
            transform: Transform {
                position: pos,
                rotation: [0.0, 0.0, 0.0],
                scale: [size, size, size],
            },
            material: MaterialData {
                color,
                metallic: 0.15,
                roughness: 0.35,
                ..Default::default()
            },
            physics: PhysicsData {
                enabled: true,
                body_type: "Dynamic".into(),
                collider: "box".into(),
                ..Default::default()
            },
            light: None,
            camera: None,
            model_path: None,
            script_path: None,
            visible: true,
        }
    }

    pub fn new_plane(id: u64, name: &str, pos: [f32; 3], size: f32, color: [f32; 4]) -> Self {
        Self {
            id,
            name: name.into(),
            kind: "plane".into(),
            transform: Transform {
                position: pos,
                rotation: [0.0, 0.0, 0.0],
                scale: [size, 1.0, size],
            },
            material: MaterialData {
                color,
                metallic: 0.05,
                roughness: 0.8,
                ..Default::default()
            },
            physics: PhysicsData {
                enabled: true,
                body_type: "Static".into(),
                collider: "box".into(),
                ..Default::default()
            },
            light: None,
            camera: None,
            model_path: None,
            script_path: None,
            visible: true,
        }
    }

    pub fn new_light(
        id: u64,
        name: &str,
        pos: [f32; 3],
        light_type: &str,
        color: [f32; 3],
        intensity: f32,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            kind: "light".into(),
            transform: Transform {
                position: pos,
                rotation: [45.0, 45.0, 0.0],
                scale: [1.0, 1.0, 1.0],
            },
            material: MaterialData {
                color: [color[0], color[1], color[2], 1.0],
                emissive: color,
                ..Default::default()
            },
            physics: PhysicsData::default(),
            light: Some(LightData {
                light_type: light_type.into(),
                color,
                intensity,
                range: 20.0,
                cast_shadows: true,
            }),
            camera: None,
            model_path: None,
            script_path: None,
            visible: true,
        }
    }

    pub fn new_camera(id: u64, name: &str, pos: [f32; 3]) -> Self {
        Self {
            id,
            name: name.into(),
            kind: "camera".into(),
            transform: Transform {
                position: pos,
                rotation: [-15.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
            },
            material: MaterialData::default(),
            physics: PhysicsData::default(),
            light: None,
            camera: Some(CameraData::default()),
            model_path: None,
            script_path: None,
            visible: true,
        }
    }
}

// Interop format matching Bevy Runtime format in templates/Runtime/src/main.rs
#[derive(Serialize, Deserialize, Default)]
pub struct BevySceneFile {
    pub name: Option<String>,
    #[serde(default)]
    pub models: Vec<BevyModelEntry>,
    #[serde(default)]
    pub objects: Vec<BevyObjectEntry>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BevyModelEntry {
    pub path: String,
    #[serde(default)]
    pub position: [f32; 3],
    #[serde(default = "default_one")]
    pub scale: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BevyObjectEntry {
    pub name: String,
    #[serde(default = "default_kind_cube")]
    pub kind: String,
    #[serde(default)]
    pub position: [f32; 3],
    #[serde(default = "default_one")]
    pub size: f32,
    #[serde(default)]
    pub color: Option<[f32; 4]>,
    #[serde(default)]
    pub physics: bool,
}

fn default_one() -> f32 {
    1.0
}
fn default_kind_cube() -> String {
    "cube".into()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub entities: Vec<Entity>,
    next_id: u64,
}

impl Default for Scene {
    fn default() -> Self {
        let mut scene = Self {
            name: "Main".into(),
            entities: Vec::new(),
            next_id: 1,
        };

        let id1 = scene.alloc_id();
        scene.add_entity(Entity::new_camera(id1, "MainCamera", [0.0, 4.0, 10.0]));
        let id2 = scene.alloc_id();
        scene.add_entity(Entity::new_light(
            id2,
            "SunLight",
            [5.0, 8.0, 5.0],
            "directional",
            [1.0, 0.95, 0.85],
            14000.0,
        ));
        let id3 = scene.alloc_id();
        scene.add_entity(Entity::new_plane(
            id3,
            "Ground",
            [0.0, 0.0, 0.0],
            20.0,
            [0.12, 0.15, 0.22, 1.0],
        ));
        let id4 = scene.alloc_id();
        scene.add_entity(Entity::new_cube(
            id4,
            "HeroCube",
            [0.0, 1.0, 0.0],
            1.8,
            [0.15, 0.45, 0.9, 1.0],
        ));

        scene
    }
}

impl Scene {
    pub fn new_empty(name: &str) -> Self {
        Self {
            name: name.into(),
            entities: Vec::new(),
            next_id: 1,
        }
    }

    pub fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add_entity(&mut self, entity: Entity) {
        if entity.id >= self.next_id {
            self.next_id = entity.id + 1;
        }
        self.entities.push(entity);
    }

    pub fn remove_entity(&mut self, id: u64) {
        self.entities.retain(|e| e.id != id);
    }

    pub fn get_entity_mut(&mut self, id: u64) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|e| e.id == id)
    }

    pub fn to_bevy_scene_file(&self) -> BevySceneFile {
        let mut file = BevySceneFile {
            name: Some(self.name.clone()),
            models: Vec::new(),
            objects: Vec::new(),
        };

        for e in &self.entities {
            if let Some(model) = &e.model_path {
                file.models.push(BevyModelEntry {
                    path: model.clone(),
                    position: e.transform.position,
                    scale: e.transform.scale[0],
                });
            } else if e.kind != "camera" && e.kind != "light" && e.kind != "empty" {
                let size = e.transform.scale[0]
                    .max(e.transform.scale[1])
                    .max(e.transform.scale[2]);
                file.objects.push(BevyObjectEntry {
                    name: e.name.clone(),
                    kind: e.kind.clone(),
                    position: e.transform.position,
                    size,
                    color: Some(e.material.color),
                    physics: e.physics.enabled,
                });
            }
        }

        file
    }

    pub fn load_from_bevy_json(&mut self, json_str: &str) -> Result<(), String> {
        let file = serde_json::from_str::<BevySceneFile>(json_str)
            .map_err(|e| format!("Falha ao decodificar cena Bevy JSON: {e}"))?;

        if let Some(n) = file.name {
            self.name = n;
        }
        self.entities.clear();
        self.next_id = 1;

        // Add default camera and light if none present in file
        let cam_id = self.alloc_id();
        self.add_entity(Entity::new_camera(cam_id, "MainCamera", [0.0, 4.0, 10.0]));
        let light_id = self.alloc_id();
        self.add_entity(Entity::new_light(
            light_id,
            "SunLight",
            [5.0, 8.0, 5.0],
            "directional",
            [1.0, 0.95, 0.85],
            14000.0,
        ));

        for obj in file.objects {
            let id = self.alloc_id();
            let c = obj.color.unwrap_or([0.2, 0.5, 0.9, 1.0]);
            let mut ent = if obj.kind == "plane" {
                Entity::new_plane(id, &obj.name, obj.position, obj.size, c)
            } else {
                let mut e = Entity::new_cube(id, &obj.name, obj.position, obj.size, c);
                e.kind = obj.kind;
                e
            };
            ent.physics.enabled = obj.physics;
            self.add_entity(ent);
        }

        for model in file.models {
            let id = self.alloc_id();
            let mut ent = Entity {
                id,
                name: format!("Model_{id}"),
                kind: "model".into(),
                transform: Transform {
                    position: model.position,
                    rotation: [0.0, 0.0, 0.0],
                    scale: [model.scale, model.scale, model.scale],
                },
                material: MaterialData::default(),
                physics: PhysicsData::default(),
                light: None,
                camera: None,
                model_path: Some(model.path),
                script_path: None,
                visible: true,
            };
            ent.name = Path::new(ent.model_path.as_ref().unwrap())
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Model")
                .to_string();
            self.add_entity(ent);
        }

        Ok(())
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        let bevy_file = self.to_bevy_scene_file();
        let json = serde_json::to_string_pretty(&bevy_file)
            .map_err(|e| format!("Erro ao serializar cena: {e}"))?;
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(path, json)
            .map_err(|e| format!("Erro ao gravar arquivo de cena '{}': {e}", path.display()))
    }

    pub fn load_from_file(&mut self, path: &Path) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Não foi possível ler '{}': {e}", path.display()))?;
        self.load_from_bevy_json(&content)
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        let mut meshes = 0;
        let mut lights = 0;
        let mut cameras = 0;
        for e in &self.entities {
            match e.kind.as_str() {
                "light" => lights += 1,
                "camera" => cameras += 1,
                _ => meshes += 1,
            }
        }
        (meshes, lights, cameras)
    }
}
