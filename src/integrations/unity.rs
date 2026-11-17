use crate::scene::{Entity, Scene};
use std::{fs, path::Path};

pub struct UnityBridge;

impl UnityBridge {
    /// Export the RevEngine scene to a Unity compatible project structure with .unity scene and .prefab files
    pub fn export_project(scene: &Scene, target_dir: &Path) -> Result<String, String> {
        let unity_dir = target_dir.join("Integrations").join("Unity");
        let assets_dir = unity_dir.join("Assets");
        let scenes_dir = assets_dir.join("Scenes");
        let settings_dir = unity_dir.join("ProjectSettings");

        fs::create_dir_all(&scenes_dir).map_err(|e| e.to_string())?;
        fs::create_dir_all(&settings_dir).map_err(|e| e.to_string())?;

        // 1. ProjectVersion.txt
        let version_txt = "m_EditorVersion: 2022.3.20f1\nm_EditorVersionWithRevision: 2022.3.20f1 (e30498b31a47)\n";
        fs::write(settings_dir.join("ProjectVersion.txt"), version_txt)
            .map_err(|e| e.to_string())?;

        // 2. Generate Main.unity YAML
        let mut yaml = String::new();
        yaml.push_str("%YAML 1.1\n%TAG !u! tag:unity3d.com,2011:\n");

        let mut file_id = 100_000u64;

        for ent in &scene.entities {
            let go_id = file_id;
            let trans_id = file_id + 1;
            file_id += 10;

            let p = ent.transform.position;
            let s = ent.transform.scale;

            yaml.push_str(&format!(
                "--- !u!1 &{}\nGameObject:\n  serializedVersion: 6\n  m_Name: {}\n  m_Component:\n  - component: {{fileID: {}}}\n",
                go_id, ent.name, trans_id
            ));

            yaml.push_str(&format!(
                "--- !u!4 &{}\nTransform:\n  m_GameObject: {{fileID: {}}}\n  m_LocalPosition: {{x: {:.3}, y: {:.3}, z: {:.3}}}\n  m_LocalScale: {{x: {:.3}, y: {:.3}, z: {:.3}}}\n  m_LocalRotation: {{x: 0, y: 0, z: 0, w: 1}}\n",
                trans_id, go_id, p[0], p[1], p[2], s[0], s[1], s[2]
            ));
        }

        let scene_file = scenes_dir.join("Main.unity");
        fs::write(&scene_file, yaml).map_err(|e| e.to_string())?;

        Ok(unity_dir.to_string_lossy().into_owned())
    }

    /// Import a Unity .unity scene or .prefab YAML file
    pub fn import_unity_yaml(content: &str, scene: &mut Scene) -> usize {
        let mut count = 0;
        let mut current_name = String::new();
        let mut current_pos = [0.0, 0.0, 0.0];
        let mut current_scale = [1.0, 1.0, 1.0];

        let flush_entity =
            |scene: &mut Scene, name: &str, pos: [f32; 3], scale: [f32; 3]| -> bool {
                if name.is_empty() || name == "Main Camera" || name == "Directional Light" {
                    return false;
                }
                let id = scene.alloc_id();
                let mut ent =
                    Entity::new_cube(id, name, pos, scale[0].max(1.0), [0.22, 0.65, 0.85, 1.0]);
                ent.transform.scale = scale;
                scene.add_entity(ent);
                true
            };

        for line in content.lines() {
            let line = line.trim();
            if let Some(name) = line.strip_prefix("m_Name: ") {
                if flush_entity(scene, &current_name, current_pos, current_scale) {
                    count += 1;
                }
                current_name = name.trim().to_string();
                current_pos = [0.0, 0.0, 0.0];
                current_scale = [1.0, 1.0, 1.0];
            } else if line.starts_with("m_LocalPosition:") {
                // e.g. m_LocalPosition: {x: 0, y: 1, z: 0}
                if let Some(inner) = line.split('{').nth(1).and_then(|s| s.split('}').next()) {
                    for part in inner.split(',') {
                        let kv: Vec<&str> = part.split(':').collect();
                        if kv.len() == 2 {
                            let val: f32 = kv[1].trim().parse().unwrap_or(0.0);
                            match kv[0].trim() {
                                "x" => current_pos[0] = val,
                                "y" => current_pos[1] = val,
                                "z" => current_pos[2] = val,
                                _ => {}
                            }
                        }
                    }
                }
            } else if line.starts_with("m_LocalScale:") {
                if let Some(inner) = line.split('{').nth(1).and_then(|s| s.split('}').next()) {
                    for part in inner.split(',') {
                        let kv: Vec<&str> = part.split(':').collect();
                        if kv.len() == 2 {
                            let val: f32 = kv[1].trim().parse().unwrap_or(1.0);
                            match kv[0].trim() {
                                "x" => current_scale[0] = val.max(0.1),
                                "y" => current_scale[1] = val.max(0.1),
                                "z" => current_scale[2] = val.max(0.1),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        if flush_entity(scene, &current_name, current_pos, current_scale) {
            count += 1;
        }

        count
    }
}
