use std::process::Command;

#[derive(Clone, Debug)]
pub struct DependencyStatus {
    pub name: String,
    pub installed: bool,
    pub version_info: String,
    pub path: Option<String>,
}

pub struct DependencyDoctor;

impl DependencyDoctor {
    pub fn check_all(blender_path: &str) -> Vec<DependencyStatus> {
        vec![
            Self::check_python(),
            Self::check_blender(blender_path),
            Self::check_cargo(),
            Self::check_godot(),
            Self::check_python_packages(),
        ]
    }

    pub fn check_python() -> DependencyStatus {
        match Command::new("python").arg("--version").output() {
            Ok(out) if out.status.success() => {
                let ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
                DependencyStatus {
                    name: "Python 3 Engine Runtime".into(),
                    installed: true,
                    version_info: if ver.is_empty() {
                        String::from_utf8_lossy(&out.stderr).trim().to_string()
                    } else {
                        ver
                    },
                    path: None,
                }
            }
            _ => DependencyStatus {
                name: "Python 3 Engine Runtime".into(),
                installed: false,
                version_info: "Não encontrado no PATH".into(),
                path: None,
            },
        }
    }

    pub fn check_blender(configured_path: &str) -> DependencyStatus {
        let is_valid = !configured_path.is_empty() && std::path::Path::new(configured_path).is_file();
        DependencyStatus {
            name: "Blender 3D Suite".into(),
            installed: is_valid,
            version_info: if is_valid {
                "Pronto para modelagem e renderização".into()
            } else {
                "Defina o caminho em Configurações".into()
            },
            path: if is_valid { Some(configured_path.to_string()) } else { None },
        }
    }

    pub fn check_cargo() -> DependencyStatus {
        match Command::new("cargo").arg("--version").output() {
            Ok(out) if out.status.success() => DependencyStatus {
                name: "Rust / Cargo Build Toolchain".into(),
                installed: true,
                version_info: String::from_utf8_lossy(&out.stdout).trim().to_string(),
                path: None,
            },
            _ => DependencyStatus {
                name: "Rust / Cargo Build Toolchain".into(),
                installed: false,
                version_info: "Não encontrado".into(),
                path: None,
            },
        }
    }

    pub fn check_godot() -> DependencyStatus {
        let mut candidates = Vec::new();

        // 1. Check PATH
        if let Ok(out) = Command::new("where.exe").arg("godot").output() {
            if out.status.success() {
                if let Some(line) = String::from_utf8_lossy(&out.stdout).lines().next() {
                    let p = line.trim();
                    if !p.is_empty() {
                        candidates.push(std::path::PathBuf::from(p));
                    }
                }
            }
        }

        // 2. Check workspace tools/godot
        candidates.push(std::path::PathBuf::from("tools").join("godot").join("godot.exe"));
        if let Ok(exe_dir) = std::env::current_exe() {
            if let Some(parent) = exe_dir.parent() {
                candidates.push(parent.join("tools").join("godot").join("godot.exe"));
                if let Some(grand) = parent.parent() {
                    candidates.push(grand.join("tools").join("godot").join("godot.exe"));
                }
            }
        }

        // 3. Check well-known directories
        if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
            candidates.push(std::path::PathBuf::from(appdata).join("Programs").join("Godot").join("godot.exe"));
        }
        candidates.push(std::path::PathBuf::from(r"C:\Program Files\Godot\godot.exe"));

        for cand in candidates {
            if cand.is_file() {
                let ver = Command::new(&cand)
                    .arg("--version")
                    .output()
                    .ok()
                    .and_then(|o| {
                        if o.status.success() {
                            Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "Godot Engine 4.x CLI".into());

                return DependencyStatus {
                    name: "Godot Engine CLI".into(),
                    installed: true,
                    version_info: format!("Instalado ({ver})"),
                    path: Some(cand.to_string_lossy().to_string()),
                };
            }
        }

        DependencyStatus {
            name: "Godot Engine CLI".into(),
            installed: false,
            version_info: "Opcional (para execução direta de .tscn)".into(),
            path: None,
        }
    }

    pub fn check_python_packages() -> DependencyStatus {
        let script = "import sys; import PIL, trimesh, numpy; print('OK')";
        match Command::new("python").args(["-c", script]).output() {
            Ok(out) if out.status.success() => DependencyStatus {
                name: "Pacotes 3D Python (trimesh, pillow, numpy)".into(),
                installed: true,
                version_info: "Todos os pacotes essenciais instalados e validados".into(),
                path: None,
            },
            _ => DependencyStatus {
                name: "Pacotes 3D Python (trimesh, pillow, numpy)".into(),
                installed: false,
                version_info: "Clique em 'Auto-Instalar Dependências' para configurar".into(),
                path: None,
            },
        }
    }

    pub fn auto_install_python_dependencies() -> Result<String, String> {
        let packages = ["pillow", "trimesh", "numpy", "watchdog"];
        let mut cmd = Command::new("python");
        cmd.args(["-m", "pip", "install", "--upgrade"]);
        for p in &packages {
            cmd.arg(p);
        }

        let output = cmd.output().map_err(|e| format!("Falha ao executar pip: {e}"))?;
        if output.status.success() {
            Ok("Dependências instaladas com sucesso: pillow, trimesh, numpy, watchdog!".into())
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            Err(format!("Erro durante a instalação: {err}"))
        }
    }
}
