use crate::scene::Scene;
use std::{fs, path::Path, process::Command};

fn python_string_literal(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".into())
}

pub struct BlenderBridge;

impl BlenderBridge {
    /// Generates a Python script that builds the current RevEngine scene inside Blender
    pub fn generate_blender_scene_script(scene: &Scene, output_glb: &Path) -> String {
        let mut py = String::new();
        py.push_str(
            r#"import bpy, math

# Clear existing objects
bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete(use_global=False)

def create_mat(name, color, metallic=0.1, roughness=0.5):
    m = bpy.data.materials.new(name)
    bsdf = m.node_tree.nodes.get("Principled BSDF")
    if bsdf:
        bsdf.inputs["Base Color"].default_value = (color[0], color[1], color[2], 1.0)
        bsdf.inputs["Metallic"].default_value = metallic
        bsdf.inputs["Roughness"].default_value = roughness
    return m

"#,
        );

        for ent in &scene.entities {
            let p = ent.transform.position;
            let s = ent.transform.scale;
            let c = ent.material.color;
            let name_clean = ent.name.replace(' ', "_");
            let py_name = python_string_literal(&name_clean);
            let py_material_name = python_string_literal(&format!("mat_{name_clean}"));

            match ent.kind.as_str() {
                "plane" => {
                    py.push_str(&format!(
                        "bpy.ops.mesh.primitive_plane_add(size={:.2}, location=({:.3}, {:.3}, {:.3}))\n",
                        s[0].max(s[2]), p[0], -p[2], p[1]
                    ));
                    py.push_str(&format!(
                        "obj = bpy.context.object; obj.name = {}\n",
                        py_name
                    ));
                    py.push_str(&format!(
                        "obj.data.materials.append(create_mat({}, [{:.3}, {:.3}, {:.3}], {:.2}, {:.2}))\n\n",
                        py_material_name, c[0], c[1], c[2], ent.material.metallic, ent.material.roughness
                    ));
                }

                "sphere" => {
                    py.push_str(&format!(
                        "bpy.ops.mesh.primitive_uv_sphere_add(radius={:.2}, location=({:.3}, {:.3}, {:.3}))\n",
                        s[0] * 0.5, p[0], -p[2], p[1]
                    ));
                    py.push_str(&format!(
                        "obj = bpy.context.object; obj.name = {}\n",
                        py_name
                    ));
                    py.push_str(&format!(
                        "obj.data.materials.append(create_mat({}, [{:.3}, {:.3}, {:.3}], {:.2}, {:.2}))\n\n",
                        py_material_name, c[0], c[1], c[2], ent.material.metallic, ent.material.roughness
                    ));
                }

                "light" => {
                    py.push_str(&format!(
                        "bpy.ops.object.light_add(type='SUN', location=({:.3}, {:.3}, {:.3}))\n",
                        p[0], -p[2], p[1]
                    ));
                    py.push_str(&format!(
                        "obj = bpy.context.object; obj.name = {}; obj.data.energy = 5.0\n\n",
                        py_name
                    ));
                }

                "camera" => {
                    py.push_str(&format!(
                        "bpy.ops.object.camera_add(location=({:.3}, {:.3}, {:.3}))\n",
                        p[0], -p[2], p[1]
                    ));
                    py.push_str(&format!(
                        "obj = bpy.context.object; obj.name = {}; obj.rotation_euler = (math.radians(68), 0.0, 0.0); bpy.context.scene.camera = obj\n\n",
                        py_name
                    ));
                }

                _ => {
                    // Default Cube / Box
                    py.push_str(&format!(
                        "bpy.ops.mesh.primitive_cube_add(size=1.0, location=({:.3}, {:.3}, {:.3}))\n",
                        p[0], -p[2], p[1]
                    ));
                    py.push_str(&format!(
                        "obj = bpy.context.object; obj.name = {}\nobj.scale = ({:.3}, {:.3}, {:.3})\n",
                        py_name, s[0], s[2], s[1]
                    ));
                    py.push_str(&format!(
                        "obj.data.materials.append(create_mat({}, [{:.3}, {:.3}, {:.3}], {:.2}, {:.2}))\n\n",
                        py_material_name, c[0], c[1], c[2], ent.material.metallic, ent.material.roughness
                    ));
                }
            }
        }

        let glb_path_str = output_glb.to_string_lossy().replace('\\', "/");
        let py_glb_path = python_string_literal(&glb_path_str);
        let py_log_path = python_string_literal(&glb_path_str);
        py.push_str(&format!(
            "bpy.ops.export_scene.gltf(filepath={}, export_format='GLB')\nprint('◈ RevEngine GLB export completed: ' + {})\n",
            py_glb_path, py_log_path
        ));

        let render_path = output_glb.with_file_name("blender_scene_render.png");
        let render_path_str = render_path.to_string_lossy().replace('\\', "/");
        let py_render_path = python_string_literal(&render_path_str);
        py.push_str(&format!(
            "scene = bpy.context.scene\nif scene.camera is None:\n    bpy.ops.object.camera_add(location=(8.0, -11.0, 7.0))\n    scene.camera = bpy.context.object\n    scene.camera.rotation_euler = (math.radians(63), 0.0, math.radians(30))\nscene.render.engine = 'CYCLES'\nscene.cycles.device = 'CPU'\nscene.cycles.samples = 2\nscene.render.resolution_x = 480\nscene.render.resolution_y = 270\nscene.render.resolution_percentage = 70\nscene.render.image_settings.file_format = 'PNG'\nscene.render.filepath = {}\nbpy.ops.render.render(write_still=True)\nprint('◈ RevEngine preview rendered: ' + {})\n",
            py_render_path, py_render_path
        ));

        py
    }

    /// Run headless Blender to export the current scene to GLB
    pub fn sync_scene_to_blender(
        scene: &Scene,
        blender_exe: &str,
        project_dir: &Path,
    ) -> Result<String, String> {
        let output_glb = project_dir
            .join("Assets")
            .join("Generated")
            .join("blender_scene.glb");
        if let Some(parent) = output_glb.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let script = Self::generate_blender_scene_script(scene, &output_glb);
        let script_file = project_dir.join(".revengine").join("blender_sync_scene.py");
        if let Some(parent) = script_file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&script_file, script).map_err(|e| e.to_string())?;

        let status = Command::new(blender_exe)
            .args(["--background", "--python"])
            .arg(&script_file)
            .status()
            .map_err(|e| format!("Falha ao executar Blender: {e}"))?;

        if status.success() {
            Ok(output_glb.to_string_lossy().into_owned())
        } else {
            Err(format!("Blender finalizou com erro (código: {status})"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{Entity, Scene};

    #[test]
    fn generated_python_escapes_entity_names() {
        let mut scene = Scene::new_empty("QA");
        scene.add_entity(Entity::new_cube(
            1,
            "Hero\"; import os",
            [0.0, 1.0, 0.0],
            1.0,
            [0.2, 0.4, 0.8, 1.0],
        ));

        let script = BlenderBridge::generate_blender_scene_script(
            &scene,
            Path::new("C:/QA/Assets/Generated/scene.glb"),
        );

        assert!(script.contains(r#"obj.name = "Hero\";_import_os""#));
        assert!(script.contains(r#"create_mat("mat_Hero\";_import_os""#));
    }

    #[test]
    fn blender_background_export_smoke_when_configured() {
        let Ok(blender_exe) = std::env::var("REVENGINE_QA_BLENDER_EXE") else {
            return;
        };
        if !Path::new(&blender_exe).is_file() {
            return;
        }

        let project_dir =
            std::env::temp_dir().join(format!("revengine_blender_smoke_{}", std::process::id()));
        let output =
            BlenderBridge::sync_scene_to_blender(&Scene::default(), &blender_exe, &project_dir)
                .expect("Blender deve exportar a cena de teste para GLB");
        let output_path = Path::new(&output);
        assert!(output_path.is_file());
        assert!(fs::metadata(output_path).unwrap().len() > 0);
        let preview_path = output_path.with_file_name("blender_scene_render.png");
        assert!(preview_path.is_file());
        assert!(fs::metadata(preview_path).unwrap().len() > 0);
        let _ = fs::remove_dir_all(project_dir);
    }
}
