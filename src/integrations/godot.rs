use crate::scene::{Entity, Scene};
use std::{fs, path::Path};

pub struct GodotBridge;

impl GodotBridge {
    /// Export the RevEngine scene to a Godot 4.x compatible project structure
    pub fn export_project(scene: &Scene, target_dir: &Path) -> Result<String, String> {
        let godot_dir = target_dir.join("Integrations").join("Godot");
        let scenes_dir = godot_dir.join("Scenes");
        fs::create_dir_all(&scenes_dir).map_err(|e| e.to_string())?;

        // 1. Generate project.godot
        let project_godot = format!(
            r#"; Engine configuration file.
; Generated automatically by RevEngine Studio 0.2.0

config_version=5

[application]

config/name="{}"
run/main_scene="res://Scenes/Main.tscn"
config/features=PackedStringArray("4.3", "Forward Plus")
config/icon="res://icon.svg"

[rendering]

anti_aliasing/quality/msaa_3d=2
"#,
            scene.name
        );
        fs::write(godot_dir.join("project.godot"), project_godot).map_err(|e| e.to_string())?;

        // 2. Generate Main.tscn
        let mut tscn = String::new();
        tscn.push_str("[gd_scene format=3 uid=\"uid://revengine_main_scene\"]\n\n");
        tscn.push_str("[node name=\"Main\" type=\"Node3D\"]\n\n");

        for ent in &scene.entities {
            let p = ent.transform.position;
            let s = ent.transform.scale;
            let name_clean = ent.name.replace(' ', "_");

            match ent.kind.as_str() {
                "light" => {
                    tscn.push_str(&format!(
                        "[node name=\"{}\" type=\"DirectionalLight3D\" parent=\".\"]\n",
                        name_clean
                    ));
                    tscn.push_str(&format!(
                        "transform = Transform3D(1, 0, 0, 0, 0.707107, 0.707107, 0, -0.707107, 0.707107, {:.3}, {:.3}, {:.3})\n",
                        p[0], p[1], p[2]
                    ));
                    if let Some(light) = &ent.light {
                        tscn.push_str(&format!(
                            "light_color = Color({:.3}, {:.3}, {:.3}, 1)\nlight_energy = {:.1}\nshadow_enabled = {}\n",
                            light.color[0], light.color[1], light.color[2], light.intensity * 0.001, light.cast_shadows
                        ));
                    }
                    tscn.push('\n');
                }

                "camera" => {
                    tscn.push_str(&format!(
                        "[node name=\"{}\" type=\"Camera3D\" parent=\".\"]\n",
                        name_clean
                    ));
                    tscn.push_str(&format!(
                        "transform = Transform3D(1, 0, 0, 0, 0.965926, 0.258819, 0, -0.258819, 0.965926, {:.3}, {:.3}, {:.3})\n\n",
                        p[0], p[1], p[2]
                    ));
                }

                _ => {
                    let node_type = if ent.physics.enabled {
                        if ent.physics.body_type == "Static" {
                            "StaticBody3D"
                        } else {
                            "RigidBody3D"
                        }
                    } else {
                        "Node3D"
                    };

                    tscn.push_str(&format!(
                        "[node name=\"{}\" type=\"{}\" parent=\".\"]\n",
                        name_clean, node_type
                    ));
                    tscn.push_str(&format!(
                        "transform = Transform3D({:.3}, 0, 0, 0, {:.3}, 0, 0, 0, {:.3}, {:.3}, {:.3}, {:.3})\n\n",
                        s[0], s[1], s[2], p[0], p[1], p[2]
                    ));

                    // Add child MeshInstance3D
                    tscn.push_str(&format!(
                        "[node name=\"MeshInstance3D\" type=\"MeshInstance3D\" parent=\"{}\"]\n",
                        name_clean
                    ));
                    tscn.push_str("mesh = SubResource(\"BoxMesh_1\")\n\n");
                }
            }
        }

        let scene_file = scenes_dir.join("Main.tscn");
        fs::write(&scene_file, tscn).map_err(|e| e.to_string())?;

        Ok(godot_dir.to_string_lossy().into_owned())
    }

    /// Import a Godot .tscn file and extract entities into a RevEngine Scene
    pub fn import_tscn(tscn_content: &str, scene: &mut Scene) -> usize {
        let mut count = 0;
        let mut current_node_name = String::new();
        let mut current_type = String::new();
        let mut current_pos = [0.0, 0.0, 0.0];
        let mut current_scale = [1.0, 1.0, 1.0];

        let flush_node = |scene: &mut Scene, name: &str, n_type: &str, pos: [f32; 3], scale: [f32; 3]| -> bool {
            if name.is_empty() || name == "Main" || name == "MeshInstance3D" {
                return false;
            }
            let id = scene.alloc_id();
            let mut ent = match n_type {
                "DirectionalLight3D" | "OmniLight3D" | "SpotLight3D" => {
                    Entity::new_light(id, name, pos, "directional", [1.0, 0.95, 0.85], 1000.0)
                }
                "Camera3D" => Entity::new_camera(id, name, pos),
                "StaticBody3D" => {
                    let mut e = Entity::new_cube(id, name, pos, scale[0].max(1.0), [0.3, 0.4, 0.6, 1.0]);
                    e.physics.body_type = "Static".into();
                    e
                }
                _ => Entity::new_cube(id, name, pos, scale[0].max(1.0), [0.2, 0.5, 0.85, 1.0]),
            };
            ent.transform.scale = scale;
            scene.add_entity(ent);
            true
        };

        for line in tscn_content.lines() {
            let line = line.trim();
            if line.starts_with("[node ") {
                if flush_node(scene, &current_node_name, &current_type, current_pos, current_scale) {
                    count += 1;
                }
                current_node_name.clear();
                current_type.clear();
                current_pos = [0.0, 0.0, 0.0];
                current_scale = [1.0, 1.0, 1.0];

                // Parse node name and type
                for part in line.trim_matches(['[', ']']).split_whitespace() {
                    if let Some(name) = part.strip_prefix("name=\"") {
                        current_node_name = name.trim_end_matches('"').to_string();
                    } else if let Some(t) = part.strip_prefix("type=\"") {
                        current_type = t.trim_end_matches('"').to_string();
                    }
                }
            } else if line.starts_with("transform = Transform3D(") {
                let inner = line.trim_start_matches("transform = Transform3D(").trim_end_matches(')');
                let nums: Vec<f32> = inner.split(',').filter_map(|s| s.trim().parse().ok()).collect();
                if nums.len() >= 12 {
                    // Extract scale magnitude from basis vectors
                    let sx = (nums[0] * nums[0] + nums[1] * nums[1] + nums[2] * nums[2]).sqrt();
                    let sy = (nums[3] * nums[3] + nums[4] * nums[4] + nums[5] * nums[5]).sqrt();
                    let sz = (nums[6] * nums[6] + nums[7] * nums[7] + nums[8] * nums[8]).sqrt();
                    current_scale = [sx.max(0.1), sy.max(0.1), sz.max(0.1)];
                    current_pos = [nums[9], nums[10], nums[11]];
                }
            }
        }

        if flush_node(scene, &current_node_name, &current_type, current_pos, current_scale) {
            count += 1;
        }

        count
    }
}
