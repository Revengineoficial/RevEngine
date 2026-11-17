use std::{fs, path::Path};

pub const RUNTIME_CARGO_TOML: &str = include_str!("../templates/Runtime/Cargo.toml");
pub const RUNTIME_CARGO_LOCK: &str = include_str!("../templates/Runtime/Cargo.lock");
pub const RUNTIME_GITIGNORE: &str = include_str!("../templates/Runtime/.gitignore");
pub const RUNTIME_MAIN_RS: &str = include_str!("../templates/Runtime/src/main.rs");
pub const RUNTIME_README_MD: &str = include_str!("../templates/Runtime/README.md");
pub const DEFAULT_SCENE_JSON: &str = include_str!("../templates/Scenes/Main.scene.json");

pub const CAVE_PYTHON_BRIDGE: &str = r#"""
Cave Engine & Python Scripting Bridge for RevEngine
Compatible with Cave Engine Component Architecture (Uniday Studio)
"""
import sys, math

class Component:
    """Base class for Cave Engine style components."""
    def __init__(self, entity=None):
        self.entity = entity
        self.enabled = True

    def start(self):
        """Called once when entity is initialized."""
        pass

    def update(self, delta_time):
        """Called every frame."""
        pass

    def on_collision(self, other):
        """Called when a physics collision occurs."""
        pass

class Vector3:
    def __init__(self, x=0.0, y=0.0, z=0.0):
        self.x, self.y, self.z = float(x), float(y), float(z)
    def __repr__(self):
        return f"Vector3({self.x}, {self.y}, {self.z})"
    def to_list(self):
        return [self.x, self.y, self.z]

class Entity:
    def __init__(self, name="Entity", position=(0,0,0)):
        self.name = name
        self.position = Vector3(*position)
        self.rotation = Vector3(0,0,0)
        self.scale = Vector3(1,1,1)
        self.components = []

    def add_component(self, comp_class):
        comp = comp_class(self)
        self.components.append(comp)
        return comp
"#;

pub fn install_project_scaffolding(project_dir: &Path) -> Result<(), String> {
    // 1. Ensure all essential directories exist
    let dirs = [
        "Scenes",
        "Scripts",
        "Assets",
        "Assets/Models",
        "Assets/Textures",
        "Assets/Audio",
        "Assets/Generated",
        "Build",
        "Plugins",
        "Runtime",
        "Runtime/src",
        "Integrations",
        "Integrations/Godot",
        "Integrations/Unity",
        "Integrations/Unreal",
    ];

    for d in &dirs {
        let full = project_dir.join(d);
        fs::create_dir_all(&full).map_err(|e| {
            format!(
                "Não foi possível criar o diretório '{}': {}",
                full.display(),
                e
            )
        })?;
    }

    // 2. Safe file creation helper: writes if not exists, or ensures content is valid
    let write_safe = |subpath: &str, content: &str| -> Result<(), String> {
        let target = project_dir.join(subpath);
        if let Some(parent) = target.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if !target.exists() {
            fs::write(&target, content)
                .map_err(|e| format!("Falha ao gravar '{}': {}", target.display(), e))?;
        }
        Ok(())
    };

    write_safe("Runtime/Cargo.toml", RUNTIME_CARGO_TOML)?;
    write_safe("Runtime/Cargo.lock", RUNTIME_CARGO_LOCK)?;
    write_safe("Runtime/.gitignore", RUNTIME_GITIGNORE)?;
    write_safe("Runtime/src/main.rs", RUNTIME_MAIN_RS)?;
    write_safe("Runtime/README.md", RUNTIME_README_MD)?;
    write_safe("Scenes/Main.scene.json", DEFAULT_SCENE_JSON)?;
    write_safe("Scripts/cave_component.py", CAVE_PYTHON_BRIDGE)?;

    // Create a project README if none exists
    let project_readme = format!(
        "# {}\n\nCriado com RevEngine Studio.\n\n- Pressione 'Executar jogo' para rodar o runtime Bevy nativo.\n- Integre com Godot, Unity, Unreal e Blender diretamente no menu de Integrações.\n",
        project_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Projeto RevEngine")
    );
    write_safe("README.md", &project_readme)?;

    Ok(())
}
