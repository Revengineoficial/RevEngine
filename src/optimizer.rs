use crate::scene::Scene;
use std::{fs, path::Path};

#[derive(Clone, Debug, Default)]
pub struct OptimizationReport {
    pub entities_analyzed: usize,
    pub static_batches: usize,
    pub physics_shapes_optimized: usize,
    pub textures_optimized: usize,
    pub memory_freed_kb: u64,
    pub details: Vec<String>,
}

pub struct AutoOptimizer;

impl AutoOptimizer {
    pub fn optimize_all(scene: &mut Scene, project_dir: Option<&Path>) -> OptimizationReport {
        let mut report = OptimizationReport::default();
        report.entities_analyzed = scene.entities.len();

        // 1. Optimize Scene Entities
        for ent in &mut scene.entities {
            // Clean invalid or extreme scales
            for s in &mut ent.transform.scale {
                if *s <= 0.001 || s.is_nan() {
                    *s = 1.0;
                    report.details.push(format!("Normalizada escala inválida em '{}'", ent.name));
                }
            }

            // Optimize Physics: Large planes or objects named "Ground"/"Floor"/"Wall" should be Static
            let name_lower = ent.name.to_lowercase();
            if ent.kind == "plane"
                || name_lower.contains("ground")
                || name_lower.contains("chao")
                || name_lower.contains("floor")
                || name_lower.contains("wall")
                || name_lower.contains("parede")
            {
                if ent.physics.body_type != "Static" {
                    ent.physics.body_type = "Static".into();
                    report.physics_shapes_optimized += 1;
                    report.details.push(format!("Corpo rígido de '{}' otimizado para 'Static' (economiza CPU Rapier/PhysX)", ent.name));
                }
                report.static_batches += 1;
            }

            // Optimize Materials: Clamp roughness and metallic within valid PBR range
            ent.material.metallic = ent.material.metallic.clamp(0.0, 1.0);
            ent.material.roughness = ent.material.roughness.clamp(0.04, 1.0);
        }

        // 2. Optimize Assets on Disk if project_dir is provided
        if let Some(dir) = project_dir {
            let assets_dir = dir.join("Assets");
            if assets_dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&assets_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            let ext_lower = ext.to_lowercase();
                            if ext_lower == "png" || ext_lower == "jpg" || ext_lower == "jpeg" {
                                if let Ok(meta) = fs::metadata(&path) {
                                    // Flag textures larger than 4MB as optimization opportunities
                                    if meta.len() > 4 * 1024 * 1024 {
                                        report.details.push(format!(
                                            "Textura pesada detectada: {} ({:.1} MB) - Sugestão: redimensionar para 2K",
                                            entry.file_name().to_string_lossy(),
                                            meta.len() as f64 / (1024.0 * 1024.0)
                                        ));
                                        report.textures_optimized += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Clean temporary cache files in .revengine
            let cache_dir = dir.join(".revengine");
            if cache_dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&cache_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            if ext == "tmp" || ext == "log" {
                                if let Ok(meta) = fs::metadata(&path) {
                                    report.memory_freed_kb += meta.len() / 1024;
                                }
                                let _ = fs::remove_file(path);
                            }
                        }
                    }
                }
            }
        }

        report
    }
}
