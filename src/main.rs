use eframe::egui;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::BufRead,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

mod converters;
mod doctor;
mod embedded_templates;
mod integrations;
mod optimizer;
mod procedural_scenes;
mod scene;
mod viewport_3d;

use converters::UniversalConverter;
use doctor::{DependencyDoctor, DependencyStatus};
use integrations::{BlenderBridge, GodotBridge, RobloxBridge, UnityBridge, UniversalEngineExporter, UnrealBridge};
use optimizer::{AutoOptimizer, OptimizationReport};
use procedural_scenes::ProceduralSceneGenerator;
use scene::{Entity, Scene};
use viewport_3d::{TransformToolMode, ViewportRenderer};

#[derive(Clone, Serialize, Deserialize)]
pub struct SceneObject {
    pub name: String,
    pub kind: String,
    pub position: [f32; 3],
    pub visible: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub dimension: String,
    pub startup_scene: String,
    pub objects: Vec<SceneObject>,
    pub scripts: Vec<String>,
    pub assets: Vec<String>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: "Meu primeiro jogo".into(),
            version: "0.2.0".into(),
            dimension: "3D".into(),
            startup_scene: "Main".into(),
            objects: vec![
                SceneObject {
                    name: "Camera".into(),
                    kind: "Camera3D".into(),
                    position: [0.0, 4.0, 10.0],
                    visible: true,
                },
                SceneObject {
                    name: "DirectionalLight".into(),
                    kind: "Light".into(),
                    position: [5.0, 8.0, 5.0],
                    visible: true,
                },
                SceneObject {
                    name: "Cube".into(),
                    kind: "Mesh3D".into(),
                    position: [0.0, 1.0, 0.0],
                    visible: true,
                },
            ],
            scripts: vec![],
            assets: vec![],
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EngineSettings {
    pub blender_exe: String,
    pub projects_directory: String,
    pub active_providers: Vec<String>,
    #[serde(default)]
    pub provider_endpoints: HashMap<String, String>,
    #[serde(default)]
    pub provider_keys: HashMap<String, String>,
    #[serde(default)]
    pub provider_models: HashMap<String, String>,
    #[serde(default)]
    pub custom_provider_name: String,
    #[serde(default)]
    pub custom_endpoint: String,
    #[serde(default)]
    pub custom_key: String,
    #[serde(default)]
    pub custom_model: String,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            blender_exe: detect_blender().unwrap_or_default(),
            projects_directory: std::env::var("USERPROFILE")
                .map(|p| format!("{p}\\Documents\\RevEngine Projects"))
                .unwrap_or_else(|_| ".".into()),
            active_providers: vec![
                "NVIDIA".into(),
                "OpenAI / ChatGPT".into(),
                "Claude".into(),
                "DeepSeek".into(),
                "Custom / Local (LM Studio / Ollama)".into(),
            ],
            provider_endpoints: [
                (
                    "NVIDIA".into(),
                    "https://integrate.api.nvidia.com/v1".into(),
                ),
                (
                    "OpenAI / ChatGPT".into(),
                    "https://api.openai.com/v1".into(),
                ),
                ("Claude".into(), "https://api.anthropic.com/v1".into()),
                ("DeepSeek".into(), "https://api.deepseek.com/v1".into()),
                ("Custom / Local (LM Studio / Ollama)".into(), "http://localhost:1234/v1".into()),
            ]
            .into_iter()
            .collect(),
            provider_keys: [
                ("NVIDIA".into(), std::env::var("NVIDIA_API_KEY").unwrap_or_default()),
                ("OpenAI / ChatGPT".into(), std::env::var("OPENAI_API_KEY").unwrap_or_default()),
                ("Claude".into(), std::env::var("ANTHROPIC_API_KEY").unwrap_or_default()),
                ("DeepSeek".into(), std::env::var("DEEPSEEK_API_KEY").unwrap_or_default()),
                ("Custom / Local (LM Studio / Ollama)".into(), String::new()),
            ]
            .into_iter()
            .collect(),
            provider_models: [
                ("NVIDIA".into(), "nvidia/nemotron-3.5-lightning-30b-a3b".into()),
                ("OpenAI / ChatGPT".into(), "gpt-4o-mini".into()),
                ("Claude".into(), "claude-3-5-sonnet-20241022".into()),
                ("DeepSeek".into(), "deepseek-chat".into()),
                ("Custom / Local (LM Studio / Ollama)".into(), "local-model".into()),
            ]
            .into_iter()
            .collect(),
            custom_provider_name: "LM Studio / Local".into(),
            custom_endpoint: "http://localhost:1234/v1".into(),
            custom_key: String::new(),
            custom_model: "local-model".into(),
        }
    }
}

pub fn default_provider_endpoint(provider: &str) -> &'static str {
    match provider {
        "NVIDIA" => "https://integrate.api.nvidia.com/v1",
        "OpenAI / ChatGPT" => "https://api.openai.com/v1",
        "Claude" => "https://api.anthropic.com/v1",
        "DeepSeek" => "https://api.deepseek.com/v1",
        "Custom / Local (LM Studio / Ollama)" => "http://localhost:1234/v1",
        _ => "",
    }
}

pub fn settings_file() -> PathBuf {
    std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("RevEngine")
        .join("settings.json")
}

pub fn load_settings() -> EngineSettings {
    fs::read_to_string(settings_file())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_settings(settings: &EngineSettings) -> Result<(), String> {
    let path = settings_file();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn detect_blender() -> Option<String> {
    let mut candidates = Vec::<PathBuf>::new();
    if let Ok(p) = std::env::var("BLENDER_EXE") {
        candidates.push(PathBuf::from(p));
    }
    if let Ok(out) = std::process::Command::new("where.exe")
        .arg("blender")
        .output()
    {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            candidates.push(PathBuf::from(line.trim()));
        }
    }
    for base in [
        r"F:\blender\blender.exe",
        r"C:\Program Files\Blender Foundation",
        r"C:\Program Files (x86)\Blender Foundation",
        r"C:\blender\blender.exe",
        r"D:\blender\blender.exe",
    ] {
        let path = PathBuf::from(base);
        if path.is_file() {
            candidates.push(path);
        } else if path.is_dir() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    candidates.push(entry.path().join("blender.exe"));
                }
            }
        }
    }
    candidates
        .into_iter()
        .find(|p| p.is_file())
        .map(|p| p.to_string_lossy().into_owned())
}

#[derive(Clone, Debug)]
pub struct ConsoleEntry {
    pub timestamp: String,
    pub level: String, // "INFO", "REVAI", "BLENDER", "AVISO", "ERRO"
    pub message: String,
}

pub struct AiCompletedResult {
    pub display_message: String,
    pub files_saved: Vec<String>,
    pub spawned_scene: Option<Scene>,
    pub reload_scene: bool,
}

pub struct RevEngineApp {
    pub project: Project,
    pub project_path: Option<PathBuf>,
    pub selected: usize,
    pub status: String,
    pub tab: String,
    pub new_name: String,
    pub new_dimension: String,
    pub script_language: String,
    pub script_name: String,
    pub ai_provider: String,
    pub ai_key: String,
    pub provider_keys: HashMap<String, String>,
    pub ai_model: String,
    pub ai_endpoint: String,
    pub show_new: bool,
    pub show_about: bool,
    pub show_settings: bool,
    pub show_integrations: bool,
    pub show_doctor: bool,
    pub show_optimizer: bool,
    pub chat_input: String,
    pub chat_history: Vec<(String, String)>,
    pub chat_rx: Option<std::sync::mpsc::Receiver<Result<AiCompletedResult, String>>>,
    pub chat_progress_rx: Option<std::sync::mpsc::Receiver<String>>,
    pub chat_busy: bool,
    pub blender_path: String,
    pub settings: EngineSettings,
    pub available_models: Vec<String>,
    pub models_status: String,
    pub blender_preview: Option<egui::TextureHandle>,
    pub asset_search: String,
    pub selected_asset: Option<PathBuf>,
    pub asset_preview: Option<(PathBuf, egui::TextureHandle)>,

    // Advanced Engine Components
    pub active_scene: Scene,
    pub viewport: ViewportRenderer,
    pub dependency_statuses: Vec<DependencyStatus>,
    pub doctor_busy: bool,
    pub optimizer_report: Option<OptimizationReport>,
    pub search_entity: String,

    // Settings UI state
    pub settings_tab: String,
    pub show_key_visibility: HashMap<String, bool>,

    // Bottom Drawer
    pub drawer_open: bool,
    pub drawer_tab: String, // "Console", "ContentBrowser", "ScriptEditor"
    pub console_logs: Vec<ConsoleEntry>,
    pub console_filter: String,

    // Integrated Script Editor
    pub script_editor_file: String,
    pub script_editor_content: String,
    pub script_editor_status: String,

    // Playback & Simulation
    pub is_simulating: bool,
    pub simulation_start: Option<Instant>,
    pub simulation_time_sec: f32,
}

impl Default for RevEngineApp {
    fn default() -> Self {
        let settings = load_settings();
        let blender_exe = settings.blender_exe.clone();
        let mut provider_keys = settings.provider_keys.clone();

        // Ensure default providers exist in provider_keys
        for p in &settings.active_providers {
            provider_keys.entry(p.clone()).or_insert_with(String::new);
        }

        let initial_provider = "NVIDIA".to_string();
        let initial_key = provider_keys.get(&initial_provider).cloned().unwrap_or_default();
        let initial_endpoint = settings
            .provider_endpoints
            .get(&initial_provider)
            .cloned()
            .unwrap_or_else(|| default_provider_endpoint(&initial_provider).into());
        let initial_model = settings
            .provider_models
            .get(&initial_provider)
            .cloned()
            .unwrap_or_else(|| "nvidia/nemotron-3.5-lightning-30b-a3b".into());

        // Auto-initialize default project so files, assets, and scenes are live from start
        let default_proj_dir = PathBuf::from(&settings.projects_directory).join("Meu primeiro jogo");
        if !default_proj_dir.exists() {
            let _ = embedded_templates::install_project_scaffolding(&default_proj_dir);
        }
        let project_path = if default_proj_dir.is_dir() {
            Some(default_proj_dir.clone())
        } else {
            None
        };

        let mut active_scene = Scene::default();
        if let Some(p) = &project_path {
            let scene_file = p.join("Scenes").join("Main.scene.json");
            if scene_file.is_file() {
                let _ = active_scene.load_from_file(&scene_file);
            }
        }

        let mut app = Self {
            project: Project::default(),
            project_path,
            selected: 0,
            status: "RevEngine Studio 0.2.0 pronto. Projeto 'Meu primeiro jogo' ativo.".into(),
            tab: "Scene".into(),
            new_name: "Meu jogo".into(),
            new_dimension: "3D".into(),
            script_language: "Rust".into(),
            script_name: "player_controller".into(),
            ai_provider: initial_provider,
            ai_key: initial_key,
            provider_keys,
            ai_model: initial_model,
            ai_endpoint: initial_endpoint,
            show_new: false,
            show_about: false,
            show_settings: false,
            show_integrations: false,
            show_doctor: false,
            show_optimizer: false,
            chat_input: String::new(),
            chat_history: vec![(
                "RevAI".into(),
                "Olá! Sou RevAI, o copiloto autônomo de criação da RevEngine Studio.\nSua suíte 3D agora possui Viewport Interativo, Gestor de Chaves de IA, Integrações Nativas com Godot, Unity, Unreal Engine e Blender, e arquitetura Cave Engine.\nO que vamos construir hoje?".into(),
            )],
            chat_rx: None,
            chat_progress_rx: None,
            chat_busy: false,
            blender_path: blender_exe.clone(),
            settings,
            available_models: Vec::new(),
            models_status: "Modelos ainda não consultados.".into(),
            blender_preview: None,
            asset_search: String::new(),
            selected_asset: None,
            asset_preview: None,

            active_scene,
            viewport: ViewportRenderer::default(),
            dependency_statuses: DependencyDoctor::check_all(&blender_exe),
            doctor_busy: false,
            optimizer_report: None,
            search_entity: String::new(),

            settings_tab: "Chaves de IA".into(),
            show_key_visibility: HashMap::new(),

            drawer_open: true,
            drawer_tab: "Console".into(),
            console_logs: Vec::new(),
            console_filter: "Todos".into(),

            script_editor_file: "Scripts/cave_component.py".into(),
            script_editor_content: embedded_templates::CAVE_PYTHON_BRIDGE.into(),
            script_editor_status: "Arquivo padrão pronto para edição.".into(),

            is_simulating: false,
            simulation_start: None,
            simulation_time_sec: 0.0,
        };

        app.log("INFO", "RevEngine Studio iniciado com sucesso. Interface Studio Dark carregada.");
        app
    }
}

impl RevEngineApp {
    pub fn log(&mut self, level: &str, msg: &str) {
        let now = chrono_or_simple_time();
        self.console_logs.push(ConsoleEntry {
            timestamp: now,
            level: level.into(),
            message: msg.into(),
        });
        if self.console_logs.len() > 300 {
            self.console_logs.remove(0);
        }
    }

    pub fn sync_scene_to_project(&mut self) {
        self.project.objects = self
            .active_scene
            .entities
            .iter()
            .map(|e| SceneObject {
                name: e.name.clone(),
                kind: e.kind.clone(),
                position: e.transform.position,
                visible: e.visible,
            })
            .collect();
    }

    pub fn sync_project_to_scene(&mut self) {
        self.active_scene.entities.clear();
        for o in &self.project.objects {
            let id = self.active_scene.alloc_id();
            let mut ent = if o.kind == "plane" {
                Entity::new_plane(id, &o.name, o.position, 20.0, [0.15, 0.18, 0.25, 1.0])
            } else if o.kind.to_lowercase().contains("light") {
                Entity::new_light(id, &o.name, o.position, "directional", [1.0, 0.95, 0.85], 14000.0)
            } else if o.kind.to_lowercase().contains("camera") {
                Entity::new_camera(id, &o.name, o.position)
            } else {
                Entity::new_cube(id, &o.name, o.position, 1.8, [0.15, 0.45, 0.9, 1.0])
            };
            ent.visible = o.visible;
            self.active_scene.add_entity(ent);
        }
    }

    pub fn save(&mut self) {
        if self.project_path.is_none() {
            let start = Path::new(&self.settings.projects_directory);
            let start = if start.is_dir() {
                start
            } else {
                Path::new(".")
            };
            if let Some(p) = rfd::FileDialog::new().set_directory(start).pick_folder() {
                self.project_path = Some(p);
            } else {
                self.status = "Salvamento cancelado.".into();
                return;
            }
        }

        let root = match self.project_path.clone() {
            Some(r) => r,
            None => return,
        };

        if let Err(e) = embedded_templates::install_project_scaffolding(&root) {
            self.status = format!("Aviso de scaffolding ao salvar: {e}");
        }

        self.sync_scene_to_project();

        let proj_res = serde_json::to_string_pretty(&self.project)
            .map_err(|e| e.to_string())
            .and_then(|s| fs::write(root.join("project.rev"), s).map_err(|e| e.to_string()));

        let scene_file = root.join("Scenes").join(format!("{}.scene.json", self.project.startup_scene));
        let scene_res = self.active_scene.save_to_file(&scene_file);

        match (proj_res, scene_res) {
            (Ok(()), Ok(())) => {
                let msg = format!("Projeto '{}' e cena salvos com sucesso: {}", self.project.name, root.display());
                self.status = msg.clone();
                self.log("INFO", &msg);
            }
            (Err(e), _) => {
                let err = format!("Erro ao salvar project.rev: {e}");
                self.status = err.clone();
                self.log("ERRO", &err);
            }
            (_, Err(e)) => {
                let err = format!("Erro ao salvar arquivo de cena: {e}");
                self.status = err.clone();
                self.log("ERRO", &err);
            }
        }
    }

    pub fn open(&mut self) {
        let start = Path::new(&self.settings.projects_directory);
        let start = if start.is_dir() {
            start
        } else {
            Path::new(".")
        };
        if let Some(file) = rfd::FileDialog::new()
            .set_directory(start)
            .add_filter("Projeto RevEngine ou Cena", &["rev", "json", "tscn", "unity", "blend"])
            .pick_file()
        {
            self.open_path(&file);
        }
    }

    pub fn open_path(&mut self, file: &Path) {
        let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        if ext == "rev" {
            match fs::read_to_string(file)
                .ok()
                .and_then(|s| serde_json::from_str::<Project>(&s).ok())
            {
                Some(p) => {
                    self.project = p;
                    let parent = file.parent().map(Path::to_path_buf);
                    self.project_path = parent.clone();
                    self.selected = 0;

                    if let Some(root) = &parent {
                        let scene_file = root.join("Scenes").join("Main.scene.json");
                        if scene_file.is_file() {
                            let _ = self.active_scene.load_from_file(&scene_file);
                        } else {
                            self.sync_project_to_scene();
                        }
                    }
                    let msg = format!("Projeto aberto: {}", file.display());
                    self.status = msg.clone();
                    self.log("INFO", &msg);
                }
                None => {
                    self.status = "Não foi possível ler este projeto RevEngine.".into();
                    self.log("ERRO", "Falha na leitura do arquivo .rev");
                }
            }
        } else if let Some(parent) = file.parent() {
            self.project_path = Some(parent.to_path_buf());
            match UniversalConverter::convert_file(file, &mut self.active_scene, parent) {
                Ok(msg) => {
                    self.sync_scene_to_project();
                    self.status = msg.clone();
                    self.log("INFO", &msg);
                }
                Err(e) => {
                    self.status = format!("Erro ao abrir arquivo: {e}");
                    self.log("ERRO", &self.status.clone());
                }
            }
        }
    }

    pub fn create_project(&mut self) {
        let start = Path::new(&self.settings.projects_directory);
        let start = if start.is_dir() {
            start
        } else {
            Path::new(".")
        };

        if let Some(root) = rfd::FileDialog::new().set_directory(start).pick_folder() {
            let clean_name = self.new_name.trim();
            let dir = if clean_name.is_empty() {
                root
            } else {
                root.join(clean_name.replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|'], "_"))
            };

            match fs::create_dir_all(&dir) {
                Ok(()) => {
                    self.project = Project::default();
                    self.project.name = if clean_name.is_empty() {
                        dir.file_name().and_then(|n| n.to_str()).unwrap_or("Meu Jogo").to_string()
                    } else {
                        clean_name.to_string()
                    };
                    self.project.dimension = self.new_dimension.clone();
                    self.project_path = Some(dir.clone());
                    self.selected = 0;

                    if let Err(e) = embedded_templates::install_project_scaffolding(&dir) {
                        self.status = format!("Aviso ao preparar templates do projeto: {e}");
                    }

                    self.active_scene = Scene::default();
                    self.active_scene.name = self.project.startup_scene.clone();
                    let _ = self.active_scene.save_to_file(&dir.join("Scenes").join("Main.scene.json"));
                    self.sync_scene_to_project();

                    self.save();
                    let msg = format!("Projeto '{}' criado com sucesso em '{}'!", self.project.name, dir.display());
                    self.status = msg.clone();
                    self.log("INFO", &msg);
                }
                Err(e) => {
                    self.status = format!("Falha ao criar diretório do projeto: {e}");
                    self.log("ERRO", &self.status.clone());
                }
            }
        }
        self.show_new = false;
    }

    pub fn run_game(&mut self) {
        let Some(root) = self.project_path.clone() else {
            self.status = "Crie ou abra um projeto antes de executar o jogo.".into();
            return;
        };

        let runtime = root.join("Runtime");
        let manifest = runtime.join("Cargo.toml");

        if !manifest.is_file() {
            if let Err(e) = embedded_templates::install_project_scaffolding(&root) {
                self.status = format!("Falha ao instalar runtime: {e}");
                return;
            }
        }

        let _ = self.active_scene.save_to_file(&root.join("Scenes").join("Main.scene.json"));

        match std::process::Command::new("cargo")
            .args(["run", "--release", "--manifest-path"])
            .arg(&manifest)
            .current_dir(&runtime)
            .spawn()
        {
            Ok(_) => {
                self.is_simulating = true;
                self.simulation_start = Some(Instant::now());
                let msg = "Runtime iniciado: compilando e executando o jogo com Bevy + Rapier 3D.".to_string();
                self.status = msg.clone();
                self.log("INFO", &msg);
            }
            Err(e) => {
                let err = format!("Não foi possível iniciar o runtime. Verifique a instalação do Rust/Cargo: {e}");
                self.status = err.clone();
                self.log("ERRO", &err);
            }
        }
    }

    pub fn stop_simulation(&mut self) {
        self.is_simulating = false;
        self.simulation_start = None;
        self.log("INFO", "Simulação pausada / parada.");
    }

    pub fn add_object(&mut self, kind: &str) {
        let id = self.active_scene.alloc_id();
        let name = format!("{}_{id}", kind);
        let pos = [0.0, 1.0, 0.0];

        let ent = match kind {
            "Sphere" | "sphere" => {
                let mut e = Entity::new_cube(id, &name, pos, 1.5, [0.85, 0.25, 0.25, 1.0]);
                e.kind = "sphere".into();
                e
            }
            "Plane" | "plane" => Entity::new_plane(id, &name, [0.0, 0.0, 0.0], 15.0, [0.2, 0.25, 0.35, 1.0]),
            "Cylinder" | "cylinder" => {
                let mut e = Entity::new_cube(id, &name, pos, 1.5, [0.25, 0.75, 0.35, 1.0]);
                e.kind = "cylinder".into();
                e
            }
            "Capsule" | "capsule" => {
                let mut e = Entity::new_cube(id, &name, pos, 1.5, [0.85, 0.55, 0.15, 1.0]);
                e.kind = "capsule".into();
                e
            }
            "Light" | "light" => Entity::new_light(id, &name, [3.0, 6.0, 3.0], "point", [1.0, 0.9, 0.7], 5000.0),
            "Camera" | "camera" => Entity::new_camera(id, &name, [0.0, 3.0, 8.0]),
            _ => Entity::new_cube(id, &name, pos, 1.5, [0.25, 0.55, 0.95, 1.0]),
        };

        self.active_scene.add_entity(ent);
        self.sync_scene_to_project();
        self.selected = self.project.objects.len().saturating_sub(1);
        self.viewport.selected_id = Some(id);
        let msg = format!("Adicionado '{}' à cena 3D.", name);
        self.status = msg.clone();
        self.log("INFO", &msg);
    }

    pub fn import_asset(&mut self) {
        let Some(root) = self.project_path.clone() else {
            self.status = "Salve ou abra o projeto antes de importar assets.".into();
            return;
        };

        if let Some(file) = rfd::FileDialog::new()
            .add_filter(
                "Todos os Assets Suportados",
                &["glb", "gltf", "obj", "fbx", "blend", "png", "jpg", "jpeg", "tscn", "unity", "prefab", "rbxlx", "rbxm", "lua", "luau", "py"],
            )
            .pick_file()
        {
            match UniversalConverter::convert_file(&file, &mut self.active_scene, &root) {
                Ok(msg) => {
                    self.sync_scene_to_project();
                    self.status = msg.clone();
                    self.log("INFO", &msg);
                }
                Err(e) => {
                    self.status = format!("Erro ao converter asset: {e}");
                    self.log("ERRO", &self.status.clone());
                }
            }
        }
    }

    pub fn run_blender(&mut self) {
        let Some(root) = self.project_path.clone() else {
            self.status = "Salve o projeto antes de executar o bridge do Blender.".into();
            return;
        };

        if self.blender_path.is_empty() || !Path::new(&self.blender_path).is_file() {
            self.status = "Blender não encontrado. Configure o caminho em ⚙ Configurações.".into();
            return;
        }

        self.log("BLENDER", "Iniciando sincronização de cena com o Blender...");
        match BlenderBridge::sync_scene_to_blender(&self.active_scene, &self.blender_path, &root) {
            Ok(glb) => {
                let msg = format!("Blender exportou com sucesso a cena GLB em '{}'!", glb);
                self.status = msg.clone();
                self.log("BLENDER", &msg);
                self.refresh_blender_preview();
            }
            Err(e) => {
                let err = format!("Erro no Blender: {e}");
                self.status = err.clone();
                self.log("ERRO", &err);
            }
        }
    }

    pub fn refresh_blender_preview(&mut self) {
        let Some(root) = self.project_path.as_ref() else {
            return;
        };
        for name in [
            "revengine_elf_portrait.png",
            "blender_scene_render.png",
            "generated_scene_render.png",
        ] {
            let p = root.join("Assets").join("Generated").join(name);
            if p.is_file() {
                if let Ok(bytes) = fs::read(&p) {
                    if let Ok(img) = image::load_from_memory(&bytes) {
                        let rgba = img.to_rgba8();
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        let _color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_flat_samples().as_slice());
                        self.asset_search = name.to_string();
                        break;
                    }
                }
            }
        }
    }

    pub fn update_active_provider_key(&mut self) {
        if let Some(key) = self.provider_keys.get(&self.ai_provider) {
            self.ai_key = key.clone();
        }
    }

    pub fn send_chat(&mut self) {
        let prompt = self.chat_input.trim().to_string();
        if prompt.is_empty() || self.chat_busy {
            return;
        }

        self.chat_history.push(("Você".into(), prompt.clone()));
        self.log("REVAI", &format!("Prompt enviado: {prompt}"));
        self.chat_input.clear();
        self.chat_busy = true;

        let (tx, rx) = std::sync::mpsc::channel();
        let (progress_tx, progress_rx) = std::sync::mpsc::channel();
        self.chat_rx = Some(rx);
        self.chat_progress_rx = Some(progress_rx);

        let provider = self.ai_provider.clone();

        // 1. Check if key is configured in settings or in-memory
        let mut key = self.provider_keys.get(&provider).cloned().unwrap_or_default();
        if key.trim().is_empty() {
            key = self.ai_key.trim().to_string();
        }
        if key.trim().is_empty() {
            key = match provider.as_str() {
                "NVIDIA" => std::env::var("NVIDIA_API_KEY").or_else(|_| std::env::var("REVENGINE_AI_API_KEY")),
                "OpenAI / ChatGPT" => std::env::var("OPENAI_API_KEY").or_else(|_| std::env::var("REVENGINE_AI_API_KEY")),
                "Claude" => std::env::var("ANTHROPIC_API_KEY").or_else(|_| std::env::var("REVENGINE_AI_API_KEY")),
                "DeepSeek" => std::env::var("DEEPSEEK_API_KEY").or_else(|_| std::env::var("REVENGINE_AI_API_KEY")),
                _ => std::env::var("REVENGINE_AI_API_KEY"),
            }
            .unwrap_or_default();
        }

        let endpoint = self.ai_endpoint.clone();
        let model = self.ai_model.clone();
        let default_project_root = PathBuf::from(&self.settings.projects_directory).join("Meu primeiro jogo");
        if !default_project_root.exists() {
            let _ = embedded_templates::install_project_scaffolding(&default_project_root);
        }
        let root = self.project_path.clone().unwrap_or(default_project_root);
        let blender = if self.blender_path.is_empty() {
            None
        } else {
            Some(self.blender_path.clone())
        };
        let bridge = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("blender")
            .join("revengine_bridge.py");

        std::thread::spawn(move || {
            let result: Result<AiCompletedResult, String> = (|| {
                let _ = progress_tx.send("RevAI: Conectando ao modelo neural...".into());
                let client = reqwest::blocking::Client::builder()
                    .timeout(Duration::from_secs(180))
                    .build()
                    .map_err(|e| e.to_string())?;

                let system = r#"Você é RevAI, o copiloto de criação autônoma de jogos da RevEngine Studio.
Sua missão é entregar jogos e cenas 3D jogáveis completas em tempo real.
Responda com UM objeto JSON puro, sem markdown adicional:
{
  "message": "Explicação clara e detalhada do que foi criado/modificado.",
  "files": [
    {
      "path": "Scenes/Main.scene.json",
      "content": "{\"name\":\"Main\",\"models\":[],\"objects\":[{\"name\":\"Ground\",\"kind\":\"plane\",\"position\":[0.0,-0.1,0.0],\"size\":30.0,\"color\":[0.1,0.15,0.2,1.0],\"physics\":false},{\"name\":\"Player\",\"kind\":\"cube\",\"position\":[0.0,1.0,0.0],\"size\":1.8,\"color\":[0.2,0.6,0.9,1.0],\"physics\":true}]}"
    }
  ],
  "blender_actions": [
    {"action":"new_scene"},
    {"action":"add_mesh","shape":"cube","name":"Base","location":[0,0,1],"size":1.8},
    {"action":"add_material","object":"Base","name":"HeroMat","color":[0.2,0.5,0.9],"metallic":0.3,"roughness":0.2},
    {"action":"add_light","light_type":"SUN","location":[5,5,8],"power":1200},
    {"action":"export","format":"GLB","filepath":"Assets/Generated/generated_scene.glb"}
  ]
}
REGRAS FUNDAMENTAIS:
1. NUNCA gere arquivos vazios. Crie Scenes/Main.scene.json para TODA solicitação de jogo ou modificação de cenário.
2. Compatível com Bevy 0.16 + Rapier 3D no Runtime/ e compatível com Unity, Godot e Unreal nativamente.
3. Se solicitado modelagem 3D detalhada, use blender_actions para criar os objetos e exportar em GLB.
"#;

                let url = if provider == "Claude" {
                    format!("{endpoint}/messages")
                } else {
                    format!("{endpoint}/chat/completions")
                };

                let body = if provider == "Claude" {
                    serde_json::json!({
                        "model": model,
                        "max_tokens": 4096,
                        "system": system,
                        "messages": [{"role": "user", "content": prompt}]
                    })
                } else {
                    serde_json::json!({
                        "model": model,
                        "messages": [
                            {"role": "system", "content": system},
                            {"role": "user", "content": prompt}
                        ],
                        "temperature": 0.3
                    })
                };

                let mut req = client.post(&url).json(&body);
                if provider == "Claude" {
                    req = req
                        .header("x-api-key", &key)
                        .header("anthropic-version", "2023-06-01");
                } else if !key.is_empty() {
                    req = req.header("Authorization", format!("Bearer {key}"));
                }

                let response = req.send().map_err(|e| format!("Falha de conexão com provedor: {e}"))?;
                let status = response.status();
                let text = response.text().map_err(|e| format!("Falha ao ler resposta HTTP: {e}"))?;

                if !status.is_success() {
                    return Err(format!("Erro da API ({status}): {text}"));
                }

                let raw_content = if provider == "Claude" {
                    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
                    v["content"][0]["text"].as_str().unwrap_or("").to_string()
                } else {
                    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
                    v["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string()
                };

                let payload_opt = extract_json_payload(&raw_content).ok();
                let message = payload_opt
                    .as_ref()
                    .and_then(|p| p["message"].as_str())
                    .unwrap_or(&raw_content)
                    .trim()
                    .to_string();

                let mut saved_files = Vec::new();
                let mut spawned_scene: Option<Scene> = None;
                let mut reload_scene = false;

                // 1. Process files from payload if available
                if let Some(payload) = &payload_opt {
                    if let Some(files) = payload["files"].as_array() {
                        for item in files {
                            if let (Some(rel), Some(content)) = (item["path"].as_str(), item["content"].as_str()) {
                                if is_safe_relative_path(rel) {
                                    let target = root.join(rel);
                                    if let Some(parent) = target.parent() {
                                        let _ = fs::create_dir_all(parent);
                                    }
                                    if fs::write(&target, content).is_ok() {
                                        saved_files.push(rel.to_string());
                                        let _ = progress_tx.send(format!("RevAI › Gravado: {rel}"));
                                        if rel.ends_with(".scene.json") || rel.ends_with(".rev") {
                                            reload_scene = true;
                                            if let Ok(sc) = serde_json::from_str::<Scene>(content) {
                                                spawned_scene = Some(sc);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(actions) = payload["blender_actions"].as_array() {
                        if !actions.is_empty() {
                            if let Some(exe) = blender.as_ref() {
                                let _ = progress_tx.send("RevAI › Executando modelagem no Blender...".into());
                                let control = root.join(".revengine");
                                let _ = fs::create_dir_all(&control);
                                let plan = control.join("blender_plan.json");
                                if let Ok(json_bytes) = serde_json::to_vec_pretty(&serde_json::json!({"actions": actions})) {
                                    let _ = fs::write(&plan, json_bytes);
                                    if let Ok(mut child) = std::process::Command::new(exe)
                                        .args(["--background", "--python"])
                                        .arg(&bridge)
                                        .arg("--")
                                        .arg("--plan")
                                        .arg(&plan)
                                        .arg("--project")
                                        .arg(&root)
                                        .stdout(std::process::Stdio::piped())
                                        .stderr(std::process::Stdio::piped())
                                        .spawn()
                                    {
                                        if let Some(stdout) = child.stdout.take() {
                                            for line in std::io::BufReader::new(stdout).lines().flatten() {
                                                let _ = progress_tx.send(format!("Blender › {line}"));
                                            }
                                        }
                                        let _ = child.wait();
                                        reload_scene = true;
                                    }
                                }
                            }
                        }
                    }
                }

                // 2. Real-Time Procedural Scene Generator (Ensures Viewport NEVER stays empty!)
                if spawned_scene.is_none() {
                    if let Some(proc_scene) = ProceduralSceneGenerator::match_or_generate_scene(&prompt, &message) {
                        let _ = progress_tx.send(format!("RevAI › ⚡ Gerando ambiente 3D procedural: '{}' com {} entidades...", proc_scene.name, proc_scene.entities.len()));
                        let scene_dir = root.join("Scenes");
                        let _ = fs::create_dir_all(&scene_dir);
                        let scene_path = scene_dir.join("Main.scene.json");
                        if let Ok(()) = proc_scene.save_to_file(&scene_path) {
                            saved_files.push("Scenes/Main.scene.json".into());
                        }
                        spawned_scene = Some(proc_scene);
                        reload_scene = true;
                    }
                }

                // 3. Native Roblox & Universal Engine Auto-Export
                if let Some(sc) = &spawned_scene {
                    let p_lower = prompt.to_lowercase();
                    if p_lower.contains("roblox") || p_lower.contains("lua") || p_lower.contains("obby") || p_lower.contains("luau") {
                        if let Ok(rbx_path) = RobloxBridge::export_project(sc, &root) {
                            let clean_name = sc.name.replace(' ', "_");
                            saved_files.push(format!("Integrations/Roblox/{clean_name}.rbxlx"));
                            saved_files.push("Integrations/Roblox/src/server/GameManager.server.luau".into());
                            saved_files.push("Integrations/Roblox/src/client/CameraController.client.luau".into());
                            saved_files.push("Integrations/Roblox/src/shared/GameConfig.luau".into());
                            let _ = progress_tx.send(format!("RevAI › 🧱 Jogo Roblox exportado com sucesso: {rbx_path}"));
                        }
                    }

                    if p_lower.contains("engine") || p_lower.contains("todas") || p_lower.contains("oficial") || p_lower.contains("universal") {
                        let b_str = blender.as_deref().unwrap_or("");
                        if let Ok(summary) = UniversalEngineExporter::export_all_targets(sc, &root, b_str) {
                            let _ = progress_tx.send(format!("RevAI › 🌐 Projeto universal exportado para {} motores (Roblox, Godot, Unity, Unreal, C++, C#, Lua)!", summary.engines_exported.len()));
                        }
                    }
                }

                Ok(AiCompletedResult {
                    display_message: message,
                    files_saved: saved_files,
                    spawned_scene,
                    reload_scene,
                })
            })();

            let _ = tx.send(result);
        });
    }
}

fn chrono_or_simple_time() -> String {
    // Portable timestamp generator without heavy chrono dependencies
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let s = secs % 60;
    format!("{hours:02}:{mins:02}:{s:02}")
}

impl eframe::App for RevEngineApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // High-End Studio Color Palette
        let panel_bg = egui::Color32::from_rgb(18, 22, 31);
        let bar_bg = egui::Color32::from_rgb(13, 16, 23);
        let accent = egui::Color32::from_rgb(88, 145, 255);
        let success = egui::Color32::from_rgb(52, 211, 153);
        let warn = egui::Color32::from_rgb(251, 191, 36);

        // Simulation timer
        if self.is_simulating {
            if let Some(start) = self.simulation_start {
                self.simulation_time_sec = start.elapsed().as_secs_f32();
                ctx.request_repaint_after(Duration::from_millis(100));
            }
        }

        // 1. Process streaming progress from RevAI
        while let Some(prog) = self.chat_progress_rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
            if let Some((_, last_msg)) = self.chat_history.last_mut() {
                last_msg.push_str("\n");
                last_msg.push_str(&prog);
            }
            self.log("REVAI", &prog);
            ctx.request_repaint();
        }

        // 2. Process finished AI response and REAL-TIME SCENE HOT-RELOAD
        if let Some(result) = self.chat_rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.chat_busy = false;
            match result {
                Ok(ai_res) => {
                    let mut full_text = ai_res.display_message;
                    if !ai_res.files_saved.is_empty() {
                        full_text.push_str("\n\nArquivos atualizados:\n");
                        for f in &ai_res.files_saved {
                            full_text.push_str(&format!("• {f}\n"));
                        }
                    }

                    // REAL-TIME SCENE SYNC & IMMEDIATE VIEWPORT INJECTION
                    if let Some(new_sc) = ai_res.spawned_scene {
                        self.active_scene = new_sc;
                        self.sync_scene_to_project();
                        if self.project_path.is_none() {
                            self.project_path = Some(PathBuf::from(&self.settings.projects_directory).join("Meu primeiro jogo"));
                        }
                        if let Some(root) = &self.project_path {
                            let _ = self.active_scene.save_to_file(&root.join("Scenes").join("Main.scene.json"));
                        }
                        self.status = format!("⚡ Cena 3D atualizada em tempo real: {} objetos gerados!", self.active_scene.entities.len());
                        self.log("REVAI", &format!("Cena 3D atualizada no Viewport com {} entidades ativas.", self.active_scene.entities.len()));
                        self.refresh_blender_preview();
                    } else if ai_res.reload_scene {
                        if let Some(root) = &self.project_path {
                            let scene_path = root.join("Scenes").join("Main.scene.json");
                            if scene_path.is_file() {
                                if let Ok(()) = self.active_scene.load_from_file(&scene_path) {
                                    self.sync_scene_to_project();
                                    self.status = "⚡ Cena 3D atualizada em tempo real pela RevAI!".into();
                                    self.log("REVAI", "Cena 3D atualizada em tempo real no Viewport.");
                                }
                            }
                        }
                        self.refresh_blender_preview();
                    }

                    self.chat_history.push(("RevAI".into(), full_text));
                }
                Err(err) => {
                    self.chat_history.push(("RevAI [Erro]".into(), format!("Falha na execução: {err}")));
                    self.status = format!("Erro RevAI: {err}");
                    self.log("ERRO", &err);
                }
            }
        }

        // 3. Top Menu Panel (Studio Header with Simulation Controls)
        egui::TopBottomPanel::top("studio_top_header")
            .frame(egui::Frame::default().fill(bar_bg).inner_margin(8.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(
                        egui::RichText::new("◈ REVENGINE STUDIO")
                            .color(accent)
                            .strong()
                            .size(19.0),
                    );
                    ui.separator();

                    if ui.button("＋ Novo").clicked() {
                        self.show_new = true;
                    }
                    if ui.button("📂 Abrir").clicked() {
                        self.open();
                    }
                    if ui.button("💾 Salvar").clicked() {
                        self.save();
                    }
                    ui.separator();

                    // --- SIMULATION CONTROLS ---
                    if !self.is_simulating {
                        if ui.button(egui::RichText::new("▶ Play").color(success).strong()).clicked() {
                            self.run_game();
                        }
                    } else {
                        if ui.button(egui::RichText::new("⏹ Stop").color(egui::Color32::from_rgb(240, 80, 80)).strong()).clicked() {
                            self.stop_simulation();
                        }
                    }

                    if self.is_simulating {
                        ui.label(
                            egui::RichText::new(format!("● SIMULANDO: {:.1}s", self.simulation_time_sec))
                                .color(success)
                                .monospace(),
                        );
                    }
                    ui.separator();

                    // Tool Modes
                    ui.selectable_value(&mut self.viewport.transform_mode, TransformToolMode::Translate, "✥ Mover (W)");
                    ui.selectable_value(&mut self.viewport.transform_mode, TransformToolMode::Rotate, "⟳ Girar (E)");
                    ui.selectable_value(&mut self.viewport.transform_mode, TransformToolMode::Scale, "⤢ Escala (R)");
                    ui.separator();

                    // Main Docks & Dialogs
                    if ui.button("🌐 Integrações").clicked() {
                        self.show_integrations = true;
                    }
                    if ui.button("⚡ Otimizador").clicked() {
                        self.show_optimizer = true;
                        let mut scene_clone = self.active_scene.clone();
                        let report = AutoOptimizer::optimize_all(&mut scene_clone, self.project_path.as_deref());
                        self.active_scene = scene_clone;
                        self.sync_scene_to_project();
                        self.optimizer_report = Some(report);
                    }
                    if ui.button("🩺 Doctor").clicked() {
                        self.dependency_statuses = DependencyDoctor::check_all(&self.blender_path);
                        self.show_doctor = true;
                    }
                    if ui.button("Blender ▶").clicked() {
                        self.run_blender();
                    }
                    if ui.button("⚙ Configurações").clicked() {
                        self.show_settings = true;
                    }

                    // Breadcrumb Right aligned
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Sobre").clicked() {
                            self.show_about = true;
                        }
                        ui.label(
                            egui::RichText::new(format!("{} › {}", self.project.name, self.active_scene.name))
                                .small()
                                .color(egui::Color32::from_gray(160)),
                        );
                    });
                });
            });

        // 4. Bottom Status & Work Drawer Toggle
        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame::default().fill(bar_bg).inner_margin(5.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.small_button(if self.drawer_open { "▼ Ocultar Gaveta" } else { "▲ Console & Gaveta" }).clicked() {
                        self.drawer_open = !self.drawer_open;
                    }
                    ui.separator();
                    ui.label("●");
                    ui.label(&self.status);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!(
                            "Grade: {}  |  Câmera: {:.1}x  |  Entidades: {}  |  {}",
                            if self.viewport.show_grid { "Ligada" } else { "Desligada" },
                            self.viewport.camera_speed,
                            self.active_scene.entities.len(),
                            self.project.dimension
                        ));
                    });
                });
            });

        // 5. Bottom Work Drawer (Console, Content Browser, Script Editor)
        if self.drawer_open {
            egui::TopBottomPanel::bottom("work_drawer")
                .resizable(true)
                .default_height(180.0)
                .min_height(120.0)
                .frame(egui::Frame::default().fill(panel_bg).inner_margin(8.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.drawer_tab, "Console".into(), "📜 Console & Logs");
                        ui.selectable_value(&mut self.drawer_tab, "ContentBrowser".into(), "▧ Content Browser (Assets)");
                        ui.selectable_value(&mut self.drawer_tab, "ScriptEditor".into(), "⌘ Editor de Scripts (Python / Rust)");

                        if self.drawer_tab == "Console" {
                            ui.separator();
                            ui.label("Filtro:");
                            ui.selectable_value(&mut self.console_filter, "Todos".into(), "Todos");
                            ui.selectable_value(&mut self.console_filter, "INFO".into(), "Info");
                            ui.selectable_value(&mut self.console_filter, "REVAI".into(), "RevAI");
                            ui.selectable_value(&mut self.console_filter, "BLENDER".into(), "Blender");
                            ui.selectable_value(&mut self.console_filter, "ERRO".into(), "Erros");

                            if ui.small_button("Limpar").clicked() {
                                self.console_logs.clear();
                            }
                        }
                    });
                    ui.separator();

                    match self.drawer_tab.as_str() {
                        "Console" => {
                            egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                                for entry in &self.console_logs {
                                    if self.console_filter != "Todos" && entry.level != self.console_filter {
                                        continue;
                                    }
                                    let badge_color = match entry.level.as_str() {
                                        "ERRO" => egui::Color32::from_rgb(240, 80, 80),
                                        "AVISO" => warn,
                                        "REVAI" => accent,
                                        "BLENDER" => egui::Color32::from_rgb(230, 140, 40),
                                        _ => success,
                                    };
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(&entry.timestamp).monospace().small().color(egui::Color32::GRAY));
                                        ui.label(egui::RichText::new(format!("[{}]", entry.level)).color(badge_color).strong());
                                        ui.label(&entry.message);
                                    });
                                }
                            });
                        }

                        "ContentBrowser" => {
                            ui.horizontal(|ui| {
                                ui.label("Pesquisar Assets:");
                                ui.text_edit_singleline(&mut self.asset_search);
                                if ui.button("Importar Novo Asset").clicked() {
                                    self.import_asset();
                                }
                                if let Some(root) = &self.project_path {
                                    if ui.button("Abrir Pasta no Explorer").clicked() {
                                        let _ = std::process::Command::new("explorer").arg(root.join("Assets")).spawn();
                                    }
                                }
                            });
                            ui.separator();

                            let mut asset_to_insert = None;
                            if let Some(root) = self.project_path.clone() {
                                let assets_dir = root.join("Assets");
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    ui.horizontal_wrapped(|ui| {
                                        if let Ok(entries) = fs::read_dir(&assets_dir) {
                                            for entry in entries.flatten() {
                                                let p = entry.path();
                                                let name = entry.file_name().to_string_lossy().to_string();
                                                let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                                                let is_dir = p.is_dir();

                                                let icon = if is_dir { "📁" } else {
                                                    match ext.as_str() {
                                                        "glb" | "gltf" | "obj" | "fbx" => "🧊",
                                                        "png" | "jpg" | "jpeg" => "🖼",
                                                        "blend" => "🔶",
                                                        _ => "📄",
                                                    }
                                                };

                                                ui.group(|ui| {
                                                    ui.vertical_centered(|ui| {
                                                        ui.label(egui::RichText::new(icon).size(24.0));
                                                        ui.label(egui::RichText::new(&name).small());
                                                        if !is_dir && ui.small_button("Inserir").clicked() {
                                                            asset_to_insert = Some((p.clone(), root.clone()));
                                                        }
                                                    });
                                                });
                                            }
                                        }
                                    });
                                });
                            } else {
                                ui.label("Salve ou abra um projeto para visualizar os arquivos de Assets.");
                            }

                            if let Some((p, root)) = asset_to_insert {
                                let _ = UniversalConverter::convert_file(&p, &mut self.active_scene, &root);
                                self.sync_scene_to_project();
                            }
                        }

                        "ScriptEditor" => {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("Editando: {}", self.script_editor_file)).strong());
                                if ui.button("💾 Salvar Script").clicked() {
                                    if let Some(root) = &self.project_path {
                                        let target = root.join(&self.script_editor_file);
                                        if let Some(parent) = target.parent() {
                                            let _ = fs::create_dir_all(parent);
                                        }
                                        match fs::write(&target, &self.script_editor_content) {
                                            Ok(()) => {
                                                self.script_editor_status = format!("Salvo em {}", target.display());
                                                self.log("INFO", &format!("Script salvo: {}", self.script_editor_file));
                                            }
                                            Err(e) => {
                                                self.script_editor_status = format!("Erro ao salvar: {e}");
                                                self.log("ERRO", &self.script_editor_status.clone());
                                            }
                                        }
                                    } else {
                                        self.script_editor_status = "Salve o projeto para salvar o script.".into();
                                    }
                                }
                                ui.label(&self.script_editor_status);
                            });
                            ui.separator();
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.script_editor_content)
                                        .font(egui::TextStyle::Monospace)
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(8),
                                );
                            });
                        }

                        _ => {}
                    }
                });
        }

        // 6. Left Hierarchy Panel
        egui::SidePanel::left("scene_hierarchy")
            .resizable(true)
            .default_width(235.0)
            .frame(egui::Frame::default().fill(panel_bg).inner_margin(8.0))
            .show(ctx, |ui| {
                ui.heading("HIERARQUIA DA CENA");
                ui.text_edit_singleline(&mut self.search_entity);

                ui.horizontal_wrapped(|ui| {
                    if ui.small_button("＋ Cubo").clicked() {
                        self.add_object("cube");
                    }
                    if ui.small_button("＋ Esfera").clicked() {
                        self.add_object("sphere");
                    }
                    if ui.small_button("＋ Plano").clicked() {
                        self.add_object("plane");
                    }
                    if ui.small_button("＋ Luz").clicked() {
                        self.add_object("light");
                    }
                    if ui.small_button("＋ Câmera").clicked() {
                        self.add_object("camera");
                    }
                });
                ui.separator();

                let search_lower = self.search_entity.to_lowercase();
                egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                    for ent in &self.active_scene.entities {
                        if !search_lower.is_empty() && !ent.name.to_lowercase().contains(&search_lower) {
                            continue;
                        }
                        let is_selected = self.viewport.selected_id == Some(ent.id);
                        let icon = match ent.kind.as_str() {
                            "light" => "☀",
                            "camera" => "📷",
                            "plane" => "▤",
                            "model" => "🧊",
                            _ => "◇",
                        };
                        let label = format!("{}  {}", icon, ent.name);
                        if ui.selectable_label(is_selected, label).clicked() {
                            self.viewport.selected_id = Some(ent.id);
                            if let Some(idx) = self.project.objects.iter().position(|o| o.name == ent.name) {
                                self.selected = idx;
                            }
                        }
                    }
                });

                ui.separator();
                ui.heading("NAVEGAÇÃO");
                if ui.selectable_label(self.drawer_tab == "ContentBrowser" && self.drawer_open, "▧ Assets do Projeto").clicked() {
                    self.drawer_open = true;
                    self.drawer_tab = "ContentBrowser".into();
                }
                if ui.selectable_label(self.drawer_tab == "ScriptEditor" && self.drawer_open, "⌘ Scripts Python / Rust").clicked() {
                    self.drawer_open = true;
                    self.drawer_tab = "ScriptEditor".into();
                }
                if ui.button("🌐 Central de Integrações").clicked() {
                    self.show_integrations = true;
                }
            });

        // 7. Right Dual Dock: Inspector & RevAI Chat
        egui::SidePanel::right("inspector_ai_dock")
            .resizable(true)
            .default_width(450.0)
            .min_width(380.0)
            .frame(egui::Frame::default().fill(panel_bg).inner_margin(8.0))
            .show(ctx, |ui| {
                // Top Half: Inspector
                ui.heading("INSPETOR");
                ui.separator();

                let mut ent_to_remove = None;
                if let Some(sel_id) = self.viewport.selected_id {
                    if let Some(ent) = self.active_scene.get_entity_mut(sel_id) {
                        ui.label("Nome da Entidade:");
                        ui.text_edit_singleline(&mut ent.name);

                        ui.horizontal(|ui| {
                            ui.label("Tipo:");
                            ui.monospace(&ent.kind);
                            ui.checkbox(&mut ent.visible, "Visível");
                        });

                        ui.separator();
                        ui.label(egui::RichText::new("Transform (Posição)").strong());
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut ent.transform.position[0]).speed(0.1).prefix("X: "));
                            ui.add(egui::DragValue::new(&mut ent.transform.position[1]).speed(0.1).prefix("Y: "));
                            ui.add(egui::DragValue::new(&mut ent.transform.position[2]).speed(0.1).prefix("Z: "));
                        });

                        ui.label(egui::RichText::new("Transform (Escala)").strong());
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut ent.transform.scale[0]).speed(0.05).prefix("SX: "));
                            ui.add(egui::DragValue::new(&mut ent.transform.scale[1]).speed(0.05).prefix("SY: "));
                            ui.add(egui::DragValue::new(&mut ent.transform.scale[2]).speed(0.05).prefix("SZ: "));
                        });

                        ui.separator();
                        ui.label(egui::RichText::new("Material PBR").strong());
                        ui.horizontal(|ui| {
                            ui.color_edit_button_rgba_unmultiplied(&mut ent.material.color);
                            ui.label("Cor Albedo");
                        });
                        ui.add(egui::Slider::new(&mut ent.material.metallic, 0.0..=1.0).text("Metálico"));
                        ui.add(egui::Slider::new(&mut ent.material.roughness, 0.0..=1.0).text("Rugosidade"));

                        ui.separator();
                        ui.label(egui::RichText::new("Física (Rapier 3D / Cave Engine)").strong());
                        ui.checkbox(&mut ent.physics.enabled, "Física Ativa");
                        if ent.physics.enabled {
                            ui.horizontal(|ui| {
                                ui.selectable_value(&mut ent.physics.body_type, "Dynamic".into(), "Dinâmico");
                                ui.selectable_value(&mut ent.physics.body_type, "Static".into(), "Estático");
                                ui.selectable_value(&mut ent.physics.body_type, "Kinematic".into(), "Cinemático");
                            });
                            ui.add(egui::Slider::new(&mut ent.physics.mass, 0.1..=100.0).text("Massa (kg)"));
                        }

                        ui.horizontal(|ui| {
                            if ui.button("Focar no Viewport (F)").clicked() {
                                self.viewport.camera.focus_on(ent.transform.position, ent.transform.scale[0]);
                            }
                            if ui.button(egui::RichText::new("Excluir").color(egui::Color32::from_rgb(255, 90, 90))).clicked() {
                                ent_to_remove = Some(sel_id);
                            }
                        });
                    }
                } else {
                    ui.label("Selecione um objeto no Viewport 3D ou na Hierarquia.");
                }

                if let Some(id) = ent_to_remove {
                    self.active_scene.remove_entity(id);
                    self.viewport.selected_id = None;
                    self.sync_scene_to_project();
                }

                ui.separator();
                // Bottom Half: RevAI Chat & Provider Status
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("✦ REVAI COPILOT").color(accent).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("⚙ Chaves").clicked() {
                            self.show_settings = true;
                            self.settings_tab = "Chaves de IA".into();
                        }
                    });
                });

                // Provider selector & Key configured badge
                let previous_p = self.ai_provider.clone();
                let providers = self.settings.active_providers.clone();
                let mut changed_provider = None;

                ui.horizontal(|ui| {
                    ui.label("Provedor:");
                    egui::ComboBox::from_id_salt("provider_select")
                        .selected_text(&self.ai_provider)
                        .show_ui(ui, |ui| {
                            for p in &providers {
                                if ui.selectable_value(&mut self.ai_provider, p.clone(), p).clicked() {
                                    if previous_p != *p {
                                        changed_provider = Some(p.clone());
                                    }
                                }
                            }
                        });

                    let has_key = !self.provider_keys.get(&self.ai_provider).cloned().unwrap_or_default().is_empty()
                        || !self.ai_key.trim().is_empty();
                    if has_key {
                        ui.label(egui::RichText::new("✔ Chave Ativa").color(success).small());
                    } else {
                        ui.label(egui::RichText::new("✖ Sem Chave").color(warn).small());
                    }
                });

                if let Some(p) = changed_provider {
                    self.update_active_provider_key();
                    if let Some(ep) = self.settings.provider_endpoints.get(&p) {
                        self.ai_endpoint = ep.clone();
                    }
                    if let Some(mdl) = self.settings.provider_models.get(&p) {
                        self.ai_model = mdl.clone();
                    }
                }

                // Chat Scroll Area
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for (sender, msg) in &self.chat_history {
                        let is_ai = sender == "RevAI";
                        ui.label(
                            egui::RichText::new(format!("{sender}:"))
                                .color(if is_ai { accent } else { egui::Color32::LIGHT_GREEN })
                                .strong(),
                        );
                        ui.label(msg);
                        ui.separator();
                    }
                });

                // Prompt Input
                ui.horizontal(|ui| {
                    let text_edit = ui.text_edit_multiline(&mut self.chat_input);
                    if ui.button("Enviar ➜").clicked()
                        || (text_edit.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl))
                    {
                        self.send_chat();
                    }
                });

                // Quick Prompt Pills
                ui.horizontal_wrapped(|ui| {
                    if ui.small_button("🧱 Jogo Roblox (Lua/Blender)").clicked() {
                        self.chat_input = "Crie um jogo completo de obby e aventura do Roblox, monte os modelos 3D no Blender e molde a lógica para Luau/Lua em pastas organizadas com arquivo .rbxlx.".into();
                        self.send_chat();
                    }
                    if ui.small_button("🏰 Castelo").clicked() {
                        self.chat_input = "Crie uma cena medieval completa com muralhas, tochas, terreno e castelo em 3D.".into();
                        self.send_chat();
                    }
                    if ui.small_button("🚀 Corrida Espacial").clicked() {
                        self.chat_input = "Crie um jogo de corrida espacial com asteroides, naves e iluminação cósmica.".into();
                        self.send_chat();
                    }
                    if ui.small_button("🧝 Elfo Fantasia").clicked() {
                        self.chat_input = "Gere um modelo 3D de elfo detalhado no Blender com render e exporte para GLB.".into();
                        self.send_chat();
                    }
                    if ui.small_button("🗺️ Terreno Cave").clicked() {
                        self.chat_input = "Adicione um terreno procedural com componentes Python compatíveis com a Cave Engine.".into();
                        self.send_chat();
                    }
                });
            });

        // 8. Central Viewport Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("VIEWPORT 3D EM TEMPO REAL");
                ui.separator();
                ui.checkbox(&mut self.viewport.show_grid, "Grade 3D");
                ui.checkbox(&mut self.viewport.wireframe_only, "Apenas Wireframe");
                ui.checkbox(&mut self.viewport.show_ground_shadows, "Sombras de Contato");
                ui.checkbox(&mut self.viewport.show_orientation_gizmo, "Bússola 3D");
                ui.separator();

                ui.label("Velocidade da Câmera:");
                ui.add(egui::Slider::new(&mut self.viewport.camera_speed, 0.5..=3.0).text("x"));

                if ui.button("Resetar Câmera").clicked() {
                    self.viewport.camera = viewport_3d::Camera3D::default();
                }
            });
            ui.separator();

            self.viewport.draw(ui, &self.active_scene, &self.blender_preview, accent);
        });

        // 9. Modal Windows
        // 9.1 Configurações Window (WITH COMPLETE API KEY CONFIGURATION FOR ALL PROVIDERS)
        if self.show_settings {
            egui::Window::new("⚙ Configurações Gerais & Chaves de Provedores")
                .default_size([680.0, 520.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.settings_tab, "Chaves de IA".into(), "🔑 Provedores de IA & Chaves");
                        ui.selectable_value(&mut self.settings_tab, "Caminhos".into(), "🛠️ Ferramentas & Caminhos");
                        ui.selectable_value(&mut self.settings_tab, "Interface".into(), "🎨 Visual & Viewport");
                    });
                    ui.separator();

                    match self.settings_tab.as_str() {
                        "Chaves de IA" => {
                            ui.heading("CONFIGURAÇÃO DE CHAVES DE API DE INTELIGÊNCIA ARTIFICIAL");
                            ui.label("Configure as chaves e modelos de cada provedor. As chaves serão salvas de forma segura no seu perfil local.");
                            ui.separator();

                            egui::ScrollArea::vertical().max_height(340.0).show(ui, |ui| {
                                for provider in [
                                    "NVIDIA",
                                    "OpenAI / ChatGPT",
                                    "Claude",
                                    "DeepSeek",
                                    "Custom / Local (LM Studio / Ollama)",
                                ] {
                                    ui.group(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.heading(provider);
                                            let key_val = self.settings.provider_keys.entry(provider.into()).or_insert_with(String::new);
                                            let is_filled = !key_val.trim().is_empty();
                                            if is_filled {
                                                ui.label(egui::RichText::new("✔ Configurada").color(success).small());
                                            } else {
                                                ui.label(egui::RichText::new("✖ Não preenchida").color(warn).small());
                                            }
                                        });

                                        // Endpoint
                                        ui.horizontal(|ui| {
                                            ui.label("Endpoint Base:");
                                            let ep = self.settings.provider_endpoints.entry(provider.into()).or_insert_with(|| default_provider_endpoint(provider).into());
                                            ui.text_edit_singleline(ep);
                                        });

                                        // Model ID
                                        ui.horizontal(|ui| {
                                            ui.label("Modelo Padrão:");
                                            let mdl = self.settings.provider_models.entry(provider.into()).or_insert_with(|| "default-model".into());
                                            ui.text_edit_singleline(mdl);
                                        });

                                        // API Key with Show/Hide toggle
                                        ui.horizontal(|ui| {
                                            ui.label("Chave de API (API Key):");
                                            let is_visible = self.show_key_visibility.entry(provider.into()).or_insert(false);
                                            let key_entry = self.settings.provider_keys.entry(provider.into()).or_insert_with(String::new);

                                            if *is_visible {
                                                ui.text_edit_singleline(key_entry);
                                            } else {
                                                ui.add(egui::TextEdit::singleline(key_entry).password(true));
                                            }

                                            if ui.small_button(if *is_visible { "Ocultar 🔒" } else { "Mostrar 👁" }).clicked() {
                                                *is_visible = !*is_visible;
                                            }
                                        });

                                        // Quick buttons
                                        ui.horizontal(|ui| {
                                            if ui.small_button("Carregar da Variável de Ambiente").clicked() {
                                                let env_val = match provider {
                                                    "NVIDIA" => std::env::var("NVIDIA_API_KEY"),
                                                    "OpenAI / ChatGPT" => std::env::var("OPENAI_API_KEY"),
                                                    "Claude" => std::env::var("ANTHROPIC_API_KEY"),
                                                    "DeepSeek" => std::env::var("DEEPSEEK_API_KEY"),
                                                    _ => std::env::var("REVENGINE_AI_API_KEY"),
                                                };
                                                if let Ok(v) = env_val {
                                                    self.settings.provider_keys.insert(provider.into(), v);
                                                }
                                            }
                                        });
                                    });
                                }
                            });
                        }

                        "Caminhos" => {
                            ui.heading("FERRAMENTAS E DIRETÓRIOS EXTERNOS");
                            ui.separator();

                            ui.label("Caminho do executável do Blender (blender.exe):");
                            ui.horizontal(|ui| {
                                ui.text_edit_singleline(&mut self.settings.blender_exe);
                                if ui.button("Auto-Detectar").clicked() {
                                    if let Some(detected) = detect_blender() {
                                        self.settings.blender_exe = detected;
                                    }
                                }
                                if ui.button("Procurar...").clicked() {
                                    if let Some(p) = rfd::FileDialog::new().add_filter("Blender", &["exe"]).pick_file() {
                                        self.settings.blender_exe = p.to_string_lossy().to_string();
                                    }
                                }
                            });

                            ui.separator();
                            ui.label("Pasta Padrão para Salvar Projetos:");
                            ui.horizontal(|ui| {
                                ui.text_edit_singleline(&mut self.settings.projects_directory);
                                if ui.button("Escolher Pasta...").clicked() {
                                    if let Some(p) = rfd::FileDialog::new().pick_folder() {
                                        self.settings.projects_directory = p.to_string_lossy().to_string();
                                    }
                                }
                            });
                        }

                        "Interface" => {
                            ui.heading("PREFERÊNCIAS VISUAIS DO STUDIO");
                            ui.separator();
                            ui.checkbox(&mut self.viewport.show_grid, "Mostrar Grade 3D no Viewport");
                            ui.checkbox(&mut self.viewport.show_orientation_gizmo, "Exibir Bússola 3D (Top-Right)");
                            ui.checkbox(&mut self.viewport.show_ground_shadows, "Exibir Sombras de Contato no Chão");
                            ui.checkbox(&mut self.viewport.wireframe_only, "Modo de Renderização Apenas Wireframe");
                        }

                        _ => {}
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("💾 Salvar Configurações").strong()).clicked() {
                            self.provider_keys = self.settings.provider_keys.clone();
                            self.blender_path = self.settings.blender_exe.clone();
                            self.update_active_provider_key();
                            let _ = save_settings(&self.settings);
                            self.show_settings = false;
                            self.status = "Configurações e chaves salvas com sucesso!".into();
                            self.log("INFO", "Configurações e chaves de API salvas com sucesso.");
                        }
                        if ui.button("Fechar").clicked() {
                            self.show_settings = false;
                        }
                    });
                });
        }

        // 9.2 Criar Projeto Window
        if self.show_new {
            egui::Window::new("Criar Novo Projeto RevEngine")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Nome do projeto:");
                    ui.text_edit_singleline(&mut self.new_name);
                    ui.label("Dimensão:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.new_dimension, "3D".into(), "3D Nativo (Bevy/OpenGL)");
                        ui.selectable_value(&mut self.new_dimension, "2D".into(), "2D");
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Escolher Pasta e Criar").clicked() {
                            self.create_project();
                        }
                        if ui.button("Cancelar").clicked() {
                            self.show_new = false;
                        }
                    });
                });
        }

        // 9.3 Integrações Window (Godot, Unity, Unreal, Blender)
        if self.show_integrations {
            egui::Window::new("🌐 Central de Integrações Nativas de Engines")
                .default_size([650.0, 480.0])
                .show(ctx, |ui| {
                    ui.heading("INTEGRAÇÕES NATIVAS COM ENGINES EXTERNAS");
                    ui.label("Exporte e sincronize cenas da RevEngine diretamente com outras engines de ponta.");
                    ui.separator();

                    if let Some(root) = self.project_path.clone() {
                        // Roblox Studio Integration (Native .rbxlx + Luau + Rojo)
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.heading("🧱 Roblox Studio (Nativo .rbxlx + Luau + Rojo)");
                                if ui.button(egui::RichText::new("Exportar para Roblox Studio (.rbxlx)").color(accent).strong()).clicked() {
                                    match RobloxBridge::export_project(&self.active_scene, &root) {
                                        Ok(p) => {
                                            self.status = format!("Exportado para Roblox Studio: {p}");
                                            self.log("INFO", &self.status.clone());
                                        }
                                        Err(e) => {
                                            self.status = format!("Erro ao exportar para Roblox: {e}");
                                            self.log("ERRO", &self.status.clone());
                                        }
                                    }
                                }
                            });
                            ui.label("Gera o arquivo oficial .rbxlx (aberto diretamente no Roblox Studio), configuração de projeto Rojo (default.project.json), scripts Luau organizados (ServerScriptService, StarterPlayerScripts, ReplicatedStorage) e pasta de assets 3D.");
                        });

                        // Universal Multi-Target Engine Exporter
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.heading("⚡ Central Universal de Exportação Multimotor");
                                if ui.button(egui::RichText::new("Exportar para TODAS as Engines (Roblox, Unity, Unreal, Godot, C++, C#, Lua)").strong()).clicked() {
                                    match UniversalEngineExporter::export_all_targets(&self.active_scene, &root, &self.blender_path) {
                                        Ok(summary) => {
                                            self.status = format!("Sucesso! Exportado para {} motores ({} arquivos) em: {}", summary.engines_exported.len(), summary.total_files, summary.output_directory);
                                            self.log("INFO", &self.status.clone());
                                        }
                                        Err(e) => {
                                            self.status = format!("Erro na exportação universal: {e}");
                                            self.log("ERRO", &self.status.clone());
                                        }
                                    }
                                }
                            });
                            ui.label("Compila a cena atual instantaneamente para Roblox (.rbxlx/Luau), Godot 4.3 (.tscn), Unity (.unity YAML), Unreal 5 (.uproject/Python/C++), C++ Nativo (OpenGL 3.3/CMake), C# (.NET 8) e Lua/Love2D!");
                        });

                        // Godot Integration
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.heading("Godot Engine 4.x");
                                if ui.button("Exportar para Godot (.tscn + project.godot)").clicked() {
                                    match GodotBridge::export_project(&self.active_scene, &root) {
                                        Ok(p) => {
                                            self.status = format!("Exportado para Godot: {p}");
                                            self.log("INFO", &self.status.clone());
                                        }
                                        Err(e) => {
                                            self.status = format!("Erro ao exportar para Godot: {e}");
                                            self.log("ERRO", &self.status.clone());
                                        }
                                    }
                                }
                            });
                            ui.label("Gera automaticamente a cena Main.tscn, nós 3D, câmeras, luzes e configuração project.godot.");
                        });

                        // Unity Integration
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.heading("Unity 2022+ / Unity 6");
                                if ui.button("Exportar para Unity (.unity + Prefabs)").clicked() {
                                    match UnityBridge::export_project(&self.active_scene, &root) {
                                        Ok(p) => {
                                            self.status = format!("Exportado para Unity: {p}");
                                            self.log("INFO", &self.status.clone());
                                        }
                                        Err(e) => {
                                            self.status = format!("Erro ao exportar para Unity: {e}");
                                            self.log("ERRO", &self.status.clone());
                                        }
                                    }
                                }
                            });
                            ui.label("Gera estrutura de projeto Unity, cena Main.unity serializada em YAML e prefabs correspondentes.");
                        });

                        // Unreal Engine Integration
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.heading("Unreal Engine 5.x");
                                if ui.button("Exportar para Unreal Engine (.uproject + Script)").clicked() {
                                    match UnrealBridge::export_project(&self.active_scene, &root) {
                                        Ok(p) => {
                                            self.status = format!("Exportado para Unreal: {p}");
                                            self.log("INFO", &self.status.clone());
                                        }
                                        Err(e) => {
                                            self.status = format!("Erro ao exportar para Unreal: {e}");
                                            self.log("ERRO", &self.status.clone());
                                        }
                                    }
                                }
                            });
                            ui.label("Gera arquivo .uproject e script Python de automação para instanciar todos os atores no Unreal Editor.");
                        });

                        // Blender Integration
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.heading("Blender 3D Suite");
                                if ui.button("Sincronizar Cena com Blender").clicked() {
                                    self.run_blender();
                                }
                            });
                            ui.label("Sincroniza todas as geometrias e materiais da cena viva com o Blender e exporta GLB otimizado.");
                        });
                    } else {
                        ui.label("Abra ou salve um projeto antes de exportar integrações.");
                    }

                    ui.separator();
                    if ui.button("Fechar").clicked() {
                        self.show_integrations = false;
                    }
                });
        }

        // 9.4 Doctor & Dependências Window
        if self.show_doctor {
            egui::Window::new("🩺 Doctor de Dependências & Ambiente")
                .default_size([550.0, 420.0])
                .show(ctx, |ui| {
                    ui.heading("DIAGNÓSTICO E AUTO-INSTALAÇÃO");
                    ui.label("Verificação automática de ferramentas e runtimes necessários.");
                    ui.separator();

                    for dep in &self.dependency_statuses {
                        ui.horizontal(|ui| {
                            if dep.installed {
                                ui.label(egui::RichText::new("✔").color(success).strong());
                            } else {
                                ui.label(egui::RichText::new("✖").color(egui::Color32::RED).strong());
                            }
                            ui.label(egui::RichText::new(&dep.name).strong());
                            ui.label(&dep.version_info);
                        });
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Auto-Instalar Dependências Python (trimesh, pillow, numpy)").clicked() {
                            match DependencyDoctor::auto_install_python_dependencies() {
                                Ok(msg) => {
                                    self.status = msg.clone();
                                    self.log("INFO", &msg);
                                    self.dependency_statuses = DependencyDoctor::check_all(&self.blender_path);
                                }
                                Err(e) => {
                                    self.status = format!("Erro na instalação: {e}");
                                    self.log("ERRO", &self.status.clone());
                                }
                            }
                        }
                        if ui.button("Reverificar Tudo").clicked() {
                            self.dependency_statuses = DependencyDoctor::check_all(&self.blender_path);
                        }
                        if ui.button("Fechar").clicked() {
                            self.show_doctor = false;
                        }
                    });
                });
        }

        // 9.5 Auto-Otimizador Window
        if self.show_optimizer {
            egui::Window::new("⚡ Relatório de Otimização Automática")
                .default_size([520.0, 360.0])
                .show(ctx, |ui| {
                    if let Some(rep) = &self.optimizer_report {
                        ui.label(format!("• Entidades analisadas: {}", rep.entities_analyzed));
                        ui.label(format!("• Corpos estáticos agrupados: {}", rep.static_batches));
                        ui.label(format!("• Formas de física otimizadas: {}", rep.physics_shapes_optimized));
                        ui.label(format!("• Texturas inspecionadas: {}", rep.textures_optimized));
                        ui.label(format!("• Memória temporária liberada: {} KB", rep.memory_freed_kb));
                        ui.separator();
                        ui.label("Detalhes das Otimizações Aplicadas:");
                        egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                            for d in &rep.details {
                                ui.label(format!("- {d}"));
                            }
                        });
                    }
                    if ui.button("Fechar").clicked() {
                        self.show_optimizer = false;
                    }
                });
        }

        // 9.6 Sobre Window
        if self.show_about {
            egui::Window::new("Sobre a RevEngine Studio")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("RevEngine Studio 0.2.0");
                    ui.label("Engine Completa de Criação de Jogos Indie com IA Integrada.");
                    ui.label("• Runtime Bevy 0.16 + Rapier 3D Nativo.");
                    ui.label("• Sincronização em Tempo Real com Blender 3D.");
                    ui.label("• Integração Nativa Bidirecional com Unity, Unreal Engine e Godot.");
                    ui.label("• Arquitetura ECS e Scripting Python inspirada na Cave Engine.");
                    ui.label("• Gestor Completo de Chaves de IA (NVIDIA, OpenAI, Claude, DeepSeek, Local).");
                    ui.separator();
                    if ui.button("Fechar").clicked() {
                        self.show_about = false;
                    }
                });
        }
    }
}

pub fn is_safe_relative_path(value: &str) -> bool {
    let path = Path::new(value);
    if value.is_empty() || path.is_absolute() {
        return false;
    }
    !path.components().any(|c| {
        matches!(
            c,
            std::path::Component::ParentDir
                | std::path::Component::RootDir
                | std::path::Component::Prefix(_)
        )
    })
}

pub fn extract_json_payload(text: &str) -> Result<serde_json::Value, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("Resposta do modelo está vazia.".into());
    }

    if let Ok(v) = serde_json::from_str(trimmed) {
        return Ok(v);
    }

    if let Some(start) = trimmed.find("```") {
        let remainder = &trimmed[start + 3..];
        let content_start = remainder.find('\n').map(|i| i + 1).unwrap_or(0);
        if let Some(end) = remainder[content_start..].find("```") {
            let candidate = &remainder[content_start..content_start + end];
            if let Ok(v) = serde_json::from_str(candidate.trim()) {
                return Ok(v);
            }
        }
    }

    if let Some(start) = trimmed.find('{') {
        let candidate = &trimmed[start..];
        let mut depth = 0;
        let mut in_string = false;
        let mut escape = false;

        for (idx, ch) in candidate.char_indices() {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == '"' {
                in_string = !in_string;
                continue;
            }
            if in_string {
                continue;
            }
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    let sub = &candidate[..=idx];
                    if let Ok(v) = serde_json::from_str(sub) {
                        return Ok(v);
                    }
                    break;
                }
            }
        }
    }

    Err("Resposta do modelo está truncada ou não contém JSON válido.".into())
}

fn configure_studio_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // Studio Dark Palette
    visuals.override_text_color = Some(egui::Color32::from_rgb(230, 235, 245));
    visuals.panel_fill = egui::Color32::from_rgb(18, 22, 31);
    visuals.window_fill = egui::Color32::from_rgb(20, 25, 36);
    visuals.window_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(45, 55, 75));
    visuals.window_corner_radius = egui::CornerRadius::same(8);

    // Widgets inactive
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(28, 34, 48);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(48, 58, 80));
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(5);

    // Widgets hovered
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(42, 52, 74);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(88, 145, 255));
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(5);

    // Widgets active
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(88, 145, 255);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(5);

    // Selection
    visuals.selection.bg_fill = egui::Color32::from_rgb(60, 110, 220);
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(100, 160, 255));

    ctx.set_visuals(visuals);
}

fn main() -> eframe::Result<()> {
    let icon_bytes = include_bytes!("../assets/branding/revengine-icon.png");
    let icon = if let Ok(image) = image::load_from_memory(icon_bytes) {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        }
    } else {
        egui::IconData::default()
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1680.0, 1000.0])
            .with_min_inner_size([1200.0, 760.0])
            .with_title("RevEngine Studio - Universal 3D Game Engine")
            .with_icon(std::sync::Arc::new(icon)),
        ..Default::default()
    };

    eframe::run_native(
        "RevEngine Studio",
        options,
        Box::new(|cc| {
            configure_studio_visuals(&cc.egui_ctx);
            Ok(Box::new(RevEngineApp::default()))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_project_relative_asset_path() {
        assert!(is_safe_relative_path("Assets/Characters/elf.glb"));
    }

    #[test]
    fn rejects_parent_directory_traversal() {
        assert!(!is_safe_relative_path("../outside.txt"));
        assert!(!is_safe_relative_path("Scripts/../../outside.txt"));
    }

    #[test]
    fn rejects_absolute_paths_and_empty_paths() {
        assert!(!is_safe_relative_path(r"C:\Windows\system.ini"));
        assert!(!is_safe_relative_path(""));
    }

    #[test]
    fn parses_json_inside_markdown_fence() {
        let value =
            extract_json_payload("```json\n{\"message\":\"ok\",\"files\":[]}\n```").unwrap();
        assert_eq!(value["message"], "ok");
    }

    #[test]
    fn extracts_json_after_explanatory_text_and_ignores_braces_in_strings() {
        let value =
            extract_json_payload("Resultado:\n{\"message\":\"chave { dentro } da string\"}")
                .unwrap();
        assert_eq!(value["message"], "chave { dentro } da string");
    }

    #[test]
    fn reports_empty_and_truncated_model_responses() {
        assert!(extract_json_payload("").unwrap_err().contains("vazia"));
        assert!(
            extract_json_payload("{\"message\":")
                .unwrap_err()
                .contains("truncada")
        );
    }

    #[test]
    fn project_starts_with_camera_and_mesh() {
        let project = Project::default();
        assert!(project.objects.iter().any(|o| o.kind == "Camera3D"));
        assert!(project.objects.iter().any(|o| o.kind == "Mesh3D"));
    }

    #[test]
    fn branding_and_blender_bridge_are_packaged() {
        assert!(!include_bytes!("../assets/branding/revengine-icon.png").is_empty());
        assert!(
            Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/blender/revengine_bridge.py"
            ))
            .exists()
        );
    }

    #[test]
    fn runtime_and_scene_templates_are_packaged() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates");
        assert!(base.join("Runtime/Cargo.toml").is_file());
        assert!(base.join("Runtime/src/main.rs").is_file());
        assert!(base.join("Scenes/Main.scene.json").is_file());
        let manifest = fs::read_to_string(base.join("Runtime/Cargo.toml")).unwrap();
        assert!(manifest.contains("bevy ="));
        assert!(manifest.contains("bevy_rapier3d"));
    }

    #[test]
    fn test_godot_export_and_import() {
        let scene = Scene::default();
        let temp_dir = std::env::temp_dir().join("revengine_test_godot");
        let export_path = GodotBridge::export_project(&scene, &temp_dir).unwrap();
        assert!(Path::new(&export_path).exists());

        let tscn_path = Path::new(&export_path).join("Scenes").join("Main.tscn");
        assert!(tscn_path.is_file());

        let content = fs::read_to_string(&tscn_path).unwrap();
        assert!(content.contains("[gd_scene"));
        assert!(content.contains("DirectionalLight3D"));

        let mut imported_scene = Scene::default();
        imported_scene.entities.clear();
        let count = GodotBridge::import_tscn(&content, &mut imported_scene);
        assert!(count > 0);
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_unity_export_and_import() {
        let scene = Scene::default();
        let temp_dir = std::env::temp_dir().join("revengine_test_unity");
        let export_path = UnityBridge::export_project(&scene, &temp_dir).unwrap();
        assert!(Path::new(&export_path).exists());

        let unity_scene = Path::new(&export_path).join("Assets").join("Scenes").join("Main.unity");
        assert!(unity_scene.is_file());

        let content = fs::read_to_string(&unity_scene).unwrap();
        assert!(content.contains("%YAML 1.1"));
        assert!(content.contains("GameObject:"));

        let mut imported_scene = Scene::default();
        imported_scene.entities.clear();
        let count = UnityBridge::import_unity_yaml(&content, &mut imported_scene);
        assert!(count > 0);
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_auto_optimizer() {
        let mut scene = Scene::default();
        let rep = AutoOptimizer::optimize_all(&mut scene, None);
        assert!(rep.entities_analyzed >= 4);
        assert!(rep.static_batches >= 1);
    }

    #[test]
    fn test_embedded_templates_scaffolding() {
        let temp_dir = std::env::temp_dir().join("revengine_test_scaffold");
        let res = embedded_templates::install_project_scaffolding(&temp_dir);
        assert!(res.is_ok());
        assert!(temp_dir.join("Runtime").join("Cargo.toml").is_file());
        assert!(temp_dir.join("Scenes").join("Main.scene.json").is_file());
        assert!(temp_dir.join("Scripts").join("cave_component.py").is_file());
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_settings_provider_keys() {
        let settings = EngineSettings::default();
        assert!(settings.provider_keys.contains_key("NVIDIA"));
        assert!(settings.provider_keys.contains_key("OpenAI / ChatGPT"));
        assert!(settings.provider_keys.contains_key("Claude"));
        assert!(settings.provider_keys.contains_key("DeepSeek"));
    }

    #[test]
    fn test_procedural_medieval_scene_generation() {
        let prompt = "Crie uma cena medieval completa com muralhas, tochas, terreno e castelo em 3D.";
        let reply = "Cenário medieval completo criado com terreno extensivo, muralhas fortificadas, torres de vigia, tochas iluminando o perímetro e um castelo central.";
        let scene = ProceduralSceneGenerator::match_or_generate_scene(prompt, reply).expect("Deve reconhecer o prompt medieval");
        assert!(scene.entities.len() >= 18);
        assert!(scene.entities.iter().any(|e| e.name.contains("Castelo") || e.name.contains("Fortaleza")));
        assert!(scene.entities.iter().any(|e| e.name.contains("Muralha")));
        assert!(scene.entities.iter().any(|e| e.name.contains("Torre")));
        assert!(scene.entities.iter().any(|e| e.name.contains("Tocha")));
        assert!(scene.entities.iter().any(|e| e.name.contains("Terreno")));
    }

    #[test]
    fn test_doctor_godot_and_python_packages() {
        let godot_stat = DependencyDoctor::check_godot();
        assert!(godot_stat.installed, "Godot CLI deve ser detectado após instalação");
        let py_stat = DependencyDoctor::check_python_packages();
        assert!(py_stat.installed, "Pacotes 3D Python (PIL, trimesh, numpy) devem ser validados");
    }

    #[test]
    fn test_roblox_bridge_export_and_import() {
        let scene = Scene::default();
        let temp_dir = std::env::temp_dir().join("revengine_test_roblox");
        let export_path = RobloxBridge::export_project(&scene, &temp_dir).unwrap();
        assert!(Path::new(&export_path).exists());
        assert!(export_path.ends_with(".rbxlx"));

        let roblox_dir = temp_dir.join("Integrations").join("Roblox");
        assert!(roblox_dir.join("default.project.json").is_file());
        assert!(roblox_dir.join("src").join("server").join("GameManager.server.luau").is_file());
        assert!(roblox_dir.join("src").join("client").join("CameraController.client.luau").is_file());
        assert!(roblox_dir.join("src").join("shared").join("GameConfig.luau").is_file());

        let content = fs::read_to_string(&export_path).unwrap();
        assert!(content.contains("<roblox"));
        assert!(content.contains("<Item class=\"Workspace\""));
        assert!(content.contains("<Item class=\"Part\""));

        let mut imported_scene = Scene::new_empty("RobloxImportTest");
        let count = RobloxBridge::import_rbxlx(&content, &mut imported_scene);
        assert!(count > 0, "Deve importar partes da cena Roblox");

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_universal_engine_exporter() {
        let scene = Scene::default();
        let temp_dir = std::env::temp_dir().join("revengine_test_universal");
        let summary = UniversalEngineExporter::export_all_targets(&scene, &temp_dir, "").unwrap();
        assert!(summary.engines_exported.len() >= 6);

        let int_dir = temp_dir.join("Integrations");
        assert!(int_dir.join("Roblox").exists());
        assert!(int_dir.join("Godot").exists());
        assert!(int_dir.join("Unity").exists());
        assert!(int_dir.join("Unreal").exists());
        assert!(int_dir.join("CppNative").join("src").join("main.cpp").is_file());
        assert!(int_dir.join("CppNative").join("CMakeLists.txt").is_file());
        assert!(int_dir.join("CSharp").join("Program.cs").is_file());
        assert!(int_dir.join("Lua").join("main.lua").is_file());

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_procedural_roblox_obby_scene() {
        let prompt = "Crie um jogo de roblox com obby, lava e trofeu";
        let reply = "Jogo do roblox criado";
        let scene = ProceduralSceneGenerator::match_or_generate_scene(prompt, reply).expect("Deve reconhecer prompt do Roblox");
        assert!(scene.entities.len() >= 12);
        assert!(scene.entities.iter().any(|e| e.name.contains("SpawnLocation")));
        assert!(scene.entities.iter().any(|e| e.name.contains("Lava")));
        assert!(scene.entities.iter().any(|e| e.name.contains("Trampolim")));
        assert!(scene.entities.iter().any(|e| e.name.contains("Trofeu")));
    }
}
