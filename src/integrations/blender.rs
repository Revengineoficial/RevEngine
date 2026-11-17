use crate::scene::Scene;
use std::{fs, path::Path, process::Command};

pub struct BlenderBridge;

impl BlenderBridge {
    /// Generates a Python script that builds the current RevEngine scene inside Blender
    pub fn generate_blender_scene_script(scene: &Scene, output_glb: &Path) -> String {
        let mut py = String::new();
        py.push_str(r#"import bpy, math

# Clear existing objects
bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete(use_global=False)

def create_mat(name, color, metallic=0.1, roughness=0.5):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    bsdf = m.node_tree.nodes.get("Principled BSDF")
    if bsdf:
        bsdf.inputs["Base Color"].default_value = (color[0], color[1], color[2], 1.0)
        bsdf.inputs["Metallic"].default_value = metallic
        bsdf.inputs["Roughness"].default_value = roughness
    return m

"#);

        for ent in &scene.entities {
            let p = ent.transform.position;
            let s = ent.transform.scale;
            let c = ent.material.color;
            let name_clean = ent.name.replace(' ', "_");

            match ent.kind.as_str() {
                "plane" => {
                    py.push_str(&format!(
                        "bpy.ops.mesh.primitive_plane_add(size={:.2}, location=({:.3}, {:.3}, {:.3}))\n",
                        s[0].max(s[2]), p[0], -p[2], p[1]
                    ));
                    py.push_str(&format!(
                        "obj = bpy.context.object; obj.name = '{}'\n",
                        name_clean
                    ));
                    py.push_str(&format!(
                        "obj.data.materials.append(create_mat('mat_{}', [{:.3}, {:.3}, {:.3}], {:.2}, {:.2}))\n\n",
                        name_clean, c[0], c[1], c[2], ent.material.metallic, ent.material.roughness
                    ));
                }

                "sphere" => {
                    py.push_str(&format!(
                        "bpy.ops.mesh.primitive_uv_sphere_add(radius={:.2}, location=({:.3}, {:.3}, {:.3}))\n",
                        s[0] * 0.5, p[0], -p[2], p[1]
                    ));
                    py.push_str(&format!(
                        "obj = bpy.context.object; obj.name = '{}'\n",
                        name_clean
                    ));
                    py.push_str(&format!(
                        "obj.data.materials.append(create_mat('mat_{}', [{:.3}, {:.3}, {:.3}], {:.2}, {:.2}))\n\n",
                        name_clean, c[0], c[1], c[2], ent.material.metallic, ent.material.roughness
                    ));
                }

                "light" => {
                    py.push_str(&format!(
                        "bpy.ops.object.light_add(type='SUN', location=({:.3}, {:.3}, {:.3}))\n",
                        p[0], -p[2], p[1]
                    ));
                    py.push_str(&format!(
                        "obj = bpy.context.object; obj.name = '{}'; obj.data.energy = 5.0\n\n",
                        name_clean
                    ));
                }

                "camera" => {
                    py.push_str(&format!(
                        "bpy.ops.object.camera_add(location=({:.3}, {:.3}, {:.3}))\n",
                        p[0], -p[2], p[1]
                    ));
                    py.push_str(&format!(
                        "obj = bpy.context.object; obj.name = '{}'\n\n",
                        name_clean
                    ));
                }

                _ => {
                    // Default Cube / Box
                    py.push_str(&format!(
                        "bpy.ops.mesh.primitive_cube_add(size=1.0, location=({:.3}, {:.3}, {:.3}))\n",
                        p[0], -p[2], p[1]
                    ));
                    py.push_str(&format!(
                        "obj = bpy.context.object; obj.name = '{}'\nobj.scale = ({:.3}, {:.3}, {:.3})\n",
                        name_clean, s[0], s[2], s[1]
                    ));
                    py.push_str(&format!(
                        "obj.data.materials.append(create_mat('mat_{}', [{:.3}, {:.3}, {:.3}], {:.2}, {:.2}))\n\n",
                        name_clean, c[0], c[1], c[2], ent.material.metallic, ent.material.roughness
                    ));
                }
            }
        }

        let glb_path_str = output_glb.to_string_lossy().replace('\\', "/");
        py.push_str(&format!(
            "bpy.ops.export_scene.gltf(filepath='{}', export_format='GLB')\nprint('◈ RevEngine GLB export completed: {}')\n",
            glb_path_str, glb_path_str
        ));

        py
    }

    /// Run headless Blender to export the current scene to GLB
    pub fn sync_scene_to_blender(
        scene: &Scene,
        blender_exe: &str,
        project_dir: &Path,
    ) -> Result<String, String> {
        let output_glb = project_dir.join("Assets").join("Generated").join("blender_scene.glb");
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
