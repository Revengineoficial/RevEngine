use crate::scene::Scene;
use std::{fs, path::Path};

pub struct UnrealBridge;

impl UnrealBridge {
    /// Export the RevEngine scene to an Unreal Engine project structure with an automation python script
    pub fn export_project(scene: &Scene, target_dir: &Path) -> Result<String, String> {
        let ue_dir = target_dir.join("Integrations").join("Unreal");
        fs::create_dir_all(&ue_dir).map_err(|e| e.to_string())?;

        // 1. Generate project.uproject
        let uproject = format!(
            r#"{{
  "FileVersion": 3,
  "EngineAssociation": "5.3",
  "Category": "",
  "Description": "RevEngine Exported Project",
  "Plugins": [
    {{
      "Name": "PythonScriptPlugin",
      "Enabled": true
    }},
    {{
      "Name": "EditorScriptingUtilities",
      "Enabled": true
    }}
  ]
}}
"#
        );
        fs::write(ue_dir.join(format!("{}.uproject", scene.name.replace(' ', "_"))), uproject)
            .map_err(|e| e.to_string())?;

        // 2. Generate Python automation script to spawn actors in Unreal Editor
        let mut py = String::new();
        py.push_str(r#"""
RevEngine to Unreal Engine 5 Scene Importer
Run this script inside Unreal Editor:
Output Log -> Cmd -> py "path/to/unreal_import_scene.py"
or via command line:
UnrealEditor-Cmd.exe "project.uproject" -ExecutePythonScript="unreal_import_scene.py"
"""
import unreal

def build_scene():
    level_lib = unreal.EditorLevelLibrary
    editor_actor_lib = unreal.EditorActorSubsystem()

    print("◈ RevEngine: Spawning scene entities in Unreal Engine...")
"#);

        for ent in &scene.entities {
            let p = ent.transform.position;
            let s = ent.transform.scale;
            // Unreal coordinate system: X is Forward, Y is Right, Z is Up (and 1 unit = 1 cm, so multiply by 100)
            let ue_x = p[2] * 100.0;
            let ue_y = p[0] * 100.0;
            let ue_z = p[1] * 100.0;

            let name_clean = ent.name.replace(' ', "_");

            match ent.kind.as_str() {
                "light" => {
                    py.push_str(&format!(
                        r#"
    # Directional Light: {}
    sun = level_lib.spawn_actor_from_class(unreal.DirectionalLight, unreal.Vector({:.1}, {:.1}, {:.1}))
    if sun:
        sun.set_actor_label("{}")
"#,
                        name_clean, ue_x, ue_y, ue_z, name_clean
                    ));
                }

                "camera" => {
                    py.push_str(&format!(
                        r#"
    # Camera: {}
    cam = level_lib.spawn_actor_from_class(unreal.CineCameraActor, unreal.Vector({:.1}, {:.1}, {:.1}))
    if cam:
        cam.set_actor_label("{}")
"#,
                        name_clean, ue_x, ue_y, ue_z, name_clean
                    ));
                }

                _ => {
                    py.push_str(&format!(
                        r#"
    # Mesh Actor: {}
    actor = level_lib.spawn_actor_from_class(unreal.StaticMeshActor, unreal.Vector({:.1}, {:.1}, {:.1}))
    if actor:
        actor.set_actor_label("{}")
        actor.set_actor_scale3d(unreal.Vector({:.2}, {:.2}, {:.2}))
"#,
                        name_clean, ue_x, ue_y, ue_z, name_clean, s[2], s[0], s[1]
                    ));
                }
            }
        }

        py.push_str("\n    print('◈ RevEngine: Cena importada com sucesso no Unreal Engine!')\n\nbuild_scene()\n");

        fs::write(ue_dir.join("unreal_import_scene.py"), py).map_err(|e| e.to_string())?;

        Ok(ue_dir.to_string_lossy().into_owned())
    }
}
