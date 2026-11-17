use crate::integrations::{godot::GodotBridge, roblox::RobloxBridge, unity::UnityBridge};
use crate::scene::{Entity, Scene};
use std::{fs, path::Path};

pub struct UniversalConverter;

impl UniversalConverter {
    /// Convert any supported file (.unity, .tscn, .blend, .fbx, .obj, .glb, .gltf, .py, .json)
    /// into native RevEngine assets and scene entities.
    pub fn convert_file(
        source_path: &Path,
        scene: &mut Scene,
        project_dir: &Path,
    ) -> Result<String, String> {
        let ext = source_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let file_stem = source_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Asset");

        match ext.as_str() {
            // 1. Godot Text Scene (.tscn)
            "tscn" => {
                let content = fs::read_to_string(source_path)
                    .map_err(|e| format!("Falha ao ler cena Godot: {e}"))?;
                let count = GodotBridge::import_tscn(&content, scene);
                Ok(format!(
                    "Cena Godot importada com sucesso: {} nós convertidos em entidades 3D da RevEngine!",
                    count
                ))
            }

            // 2. Unity Scene (.unity) or Prefab (.prefab)
            "unity" | "prefab" => {
                let content = fs::read_to_string(source_path)
                    .map_err(|e| format!("Falha ao ler arquivo Unity: {e}"))?;
                let count = UnityBridge::import_unity_yaml(&content, scene);
                Ok(format!(
                    "Arquivo Unity ({}) importado: {} GameObjects convertidos em entidades nativas!",
                    ext, count
                ))
            }

            // 2b. Roblox Studio Place (.rbxlx) or Model (.rbxm)
            "rbxlx" | "rbxm" => {
                let content = fs::read_to_string(source_path)
                    .map_err(|e| format!("Falha ao ler arquivo Roblox: {e}"))?;
                let count = RobloxBridge::import_rbxlx(&content, scene);
                Ok(format!(
                    "Cena Roblox Studio ({}) importada: {} Parts convertidas em entidades 3D da RevEngine!",
                    ext, count
                ))
            }

            // 2c. Luau / Lua Script (.lua / .luau)
            "lua" | "luau" => {
                let scripts_folder = project_dir.join("Scripts").join("Roblox");
                let _ = fs::create_dir_all(&scripts_folder);
                let dest_file = scripts_folder.join(source_path.file_name().unwrap());

                fs::copy(source_path, &dest_file)
                    .map_err(|e| format!("Falha ao copiar script Luau: {e}"))?;

                let rel_path = format!(
                    "Scripts/Roblox/{}",
                    source_path.file_name().unwrap().to_string_lossy()
                );
                Ok(format!(
                    "Script Luau / Lua '{}' importado para '{}'!",
                    file_stem, rel_path
                ))
            }

            // 3. 3D Model Formats (.glb, .gltf, .obj, .fbx)
            "glb" | "gltf" | "obj" | "fbx" => {
                let dest_folder = project_dir.join("Assets").join("Models");
                let _ = fs::create_dir_all(&dest_folder);
                let dest_file = dest_folder.join(source_path.file_name().unwrap());

                fs::copy(source_path, &dest_file)
                    .map_err(|e| format!("Falha ao copiar modelo 3D: {e}"))?;

                let rel_path = format!(
                    "Assets/Models/{}",
                    source_path.file_name().unwrap().to_string_lossy()
                );
                let id = scene.alloc_id();

                let mut ent =
                    Entity::new_cube(id, file_stem, [0.0, 1.0, 0.0], 1.0, [0.8, 0.8, 0.85, 1.0]);
                ent.kind = "model".into();
                ent.model_path = Some(rel_path.clone());
                scene.add_entity(ent);

                Ok(format!(
                    "Modelo 3D '{}' importado e adicionado à cena em '{}'!",
                    file_stem, rel_path
                ))
            }

            // 4. Blender Source File (.blend)
            "blend" => {
                let dest_folder = project_dir.join("Assets").join("Models");
                let _ = fs::create_dir_all(&dest_folder);
                let dest_file = dest_folder.join(source_path.file_name().unwrap());

                fs::copy(source_path, &dest_file)
                    .map_err(|e| format!("Falha ao importar .blend: {e}"))?;

                Ok(format!(
                    "Arquivo Blender nativo '{}' copiado para Assets/Models. Use o menu Blender para gerar renders ou GLB.",
                    source_path.file_name().unwrap().to_string_lossy()
                ))
            }

            // 5. Cave Engine / Python Script (.py)
            "py" => {
                let scripts_folder = project_dir.join("Scripts");
                let _ = fs::create_dir_all(&scripts_folder);
                let dest_file = scripts_folder.join(source_path.file_name().unwrap());

                fs::copy(source_path, &dest_file)
                    .map_err(|e| format!("Falha ao copiar script: {e}"))?;

                let rel_path = format!(
                    "Scripts/{}",
                    source_path.file_name().unwrap().to_string_lossy()
                );

                Ok(format!(
                    "Script Python / Cave Engine '{}' importado para '{}'. Compatível com componentes RevEngine!",
                    file_stem, rel_path
                ))
            }

            // 6. Native RevEngine Scene (.json / .rev)
            "json" | "rev" => {
                scene.load_from_file(source_path)?;
                Ok(format!(
                    "Cena '{}' carregada com sucesso!",
                    source_path.display()
                ))
            }

            // 7. Textures (.png, .jpg, .jpeg, .tga, .bmp)
            "png" | "jpg" | "jpeg" | "tga" | "bmp" => {
                let textures_folder = project_dir.join("Assets").join("Textures");
                let _ = fs::create_dir_all(&textures_folder);
                let dest_file = textures_folder.join(source_path.file_name().unwrap());

                fs::copy(source_path, &dest_file)
                    .map_err(|e| format!("Falha ao importar textura: {e}"))?;

                Ok(format!(
                    "Textura '{}' copiada para Assets/Textures/!",
                    source_path.file_name().unwrap().to_string_lossy()
                ))
            }

            _ => Err(format!(
                "Formato '.{}' não reconhecido para conversão automática.",
                ext
            )),
        }
    }
}
