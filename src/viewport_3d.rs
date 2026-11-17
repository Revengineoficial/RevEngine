use crate::scene::{Entity, Scene};
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

#[derive(Clone, Debug)]
pub struct Camera3D {
    pub target: [f32; 3],
    pub yaw: f32,   // radians
    pub pitch: f32, // radians
    pub distance: f32,
    pub fov_deg: f32,
    pub ortho: bool,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            target: [0.0, 1.0, 0.0],
            yaw: 0.785,  // 45 degrees
            pitch: 0.45, // ~25 degrees
            distance: 12.0,
            fov_deg: 50.0,
            ortho: false,
        }
    }
}

impl Camera3D {
    pub fn eye_position(&self) -> [f32; 3] {
        let cos_p = self.pitch.cos();
        let sin_p = self.pitch.sin();
        let cos_y = self.yaw.cos();
        let sin_y = self.yaw.sin();

        [
            self.target[0] + self.distance * cos_p * sin_y,
            self.target[1] + self.distance * sin_p,
            self.target[2] + self.distance * cos_p * cos_y,
        ]
    }

    pub fn focus_on(&mut self, pos: [f32; 3], size: f32) {
        self.target = pos;
        self.distance = (size * 3.0).clamp(4.0, 60.0);
    }

    pub fn snap_to_top(&mut self) {
        self.pitch = 1.56; // ~89 degrees
        self.yaw = 0.0;
    }

    pub fn snap_to_front(&mut self) {
        self.pitch = 0.0;
        self.yaw = 0.0;
    }

    pub fn snap_to_right(&mut self) {
        self.pitch = 0.0;
        self.yaw = std::f32::consts::FRAC_PI_2;
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TransformToolMode {
    Translate,
    Rotate,
    Scale,
}

pub struct ViewportRenderer {
    pub camera: Camera3D,
    pub wireframe_only: bool,
    pub show_grid: bool,
    pub show_stats: bool,
    pub show_orientation_gizmo: bool,
    pub show_ground_shadows: bool,
    pub transform_mode: TransformToolMode,
    pub snap_grid: f32, // 0.0 = off, 0.5, 1.0, 2.0
    pub camera_speed: f32,
    pub selected_id: Option<u64>,
    pub show_render_preview: bool,
}

impl Default for ViewportRenderer {
    fn default() -> Self {
        Self {
            camera: Camera3D::default(),
            wireframe_only: false,
            show_grid: true,
            show_stats: true,
            show_orientation_gizmo: true,
            show_ground_shadows: true,
            transform_mode: TransformToolMode::Translate,
            snap_grid: 0.0,
            camera_speed: 1.0,
            selected_id: None,
            show_render_preview: false,
        }
    }
}

struct ProjectedPoint {
    screen: Pos2,
    depth: f32,
    visible: bool,
}

impl ViewportRenderer {
    // Project 3D point (x, y, z) into 2D screen coordinates
    fn project(&self, p: [f32; 3], rect: Rect) -> ProjectedPoint {
        let eye = self.camera.eye_position();
        let dx = p[0] - eye[0];
        let dy = p[1] - eye[1];
        let dz = p[2] - eye[2];

        // Camera basis vectors
        let cos_p = self.camera.pitch.cos();
        let sin_p = self.camera.pitch.sin();
        let cos_y = self.camera.yaw.cos();
        let sin_y = self.camera.yaw.sin();

        // Forward vector (from eye towards target)
        let fwd = [-cos_p * sin_y, -sin_p, -cos_p * cos_y];
        // Right vector
        let right = [cos_y, 0.0, -sin_y];
        // Up vector
        let up = [-sin_p * sin_y, cos_p, -sin_p * cos_y];

        // Coordinates in camera space
        let cam_x = dx * right[0] + dy * right[1] + dz * right[2];
        let cam_y = dx * up[0] + dy * up[1] + dz * up[2];
        let cam_z = dx * fwd[0] + dy * fwd[1] + dz * fwd[2];

        if cam_z <= 0.1 {
            return ProjectedPoint {
                screen: Pos2::ZERO,
                depth: cam_z,
                visible: false,
            };
        }

        let fov_rad = self.camera.fov_deg.to_radians();
        let fov_scale = 1.0 / (fov_rad * 0.5).tan();

        let aspect = rect.width() / rect.height().max(1.0);
        let center = rect.center();

        let norm_x = (cam_x / cam_z) * fov_scale / aspect;
        let norm_y = (cam_y / cam_z) * fov_scale;

        let screen_x = center.x + norm_x * (rect.width() * 0.5);
        let screen_y = center.y - norm_y * (rect.height() * 0.5);

        ProjectedPoint {
            screen: Pos2::new(screen_x, screen_y),
            depth: cam_z,
            visible: true,
        }
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        scene: &Scene,
        blender_preview: &Option<egui::TextureHandle>,
        accent: Color32,
    ) {
        let (rect, response) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        // AI render preview is a dedicated viewport mode, never a translucent backdrop
        // hidden beneath the editor grid and placeholder geometry.
        if self.show_render_preview {
            if let Some(texture) = blender_preview {
                painter.image(
                    texture.id(),
                    rect,
                    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
                let pill = Rect::from_min_size(
                    rect.left_top() + Vec2::splat(12.0),
                    Vec2::new(300.0, 34.0),
                );
                painter.rect_filled(pill, 8.0, Color32::from_black_alpha(190));
                painter.text(
                    pill.center(),
                    egui::Align2::CENTER_CENTER,
                    "RENDER DA REVAI  •  DUPLO CLIQUE PARA EDITAR",
                    egui::FontId::proportional(11.0),
                    Color32::WHITE,
                );
                if response.double_clicked() {
                    self.show_render_preview = false;
                }
                return;
            }
            self.show_render_preview = false;
        }

        // 1. Draw Sleek Atmospheric Viewport Skybox
        self.draw_atmospheric_background(&painter, rect);

        // 2. Mouse Orbit / Pan / Zoom interactions
        if response.dragged_by(egui::PointerButton::Secondary) {
            let delta = response.drag_delta();
            if ui.input(|i| i.modifiers.shift) {
                // Pan
                let pan_speed = self.camera.distance * 0.0016 * self.camera_speed;
                let cos_y = self.camera.yaw.cos();
                let sin_y = self.camera.yaw.sin();
                self.camera.target[0] -= (delta.x * cos_y - delta.y * sin_y) * pan_speed;
                self.camera.target[1] += delta.y * pan_speed;
                self.camera.target[2] -= (-delta.x * sin_y - delta.y * cos_y) * pan_speed;
            } else {
                // Orbit
                self.camera.yaw -= delta.x * 0.007 * self.camera_speed;
                self.camera.pitch =
                    (self.camera.pitch + delta.y * 0.007 * self.camera_speed).clamp(-1.48, 1.48);
            }
        } else if response.dragged_by(egui::PointerButton::Middle) {
            let delta = response.drag_delta();
            let pan_speed = self.camera.distance * 0.0016 * self.camera_speed;
            self.camera.target[0] -= delta.x * pan_speed;
            self.camera.target[1] += delta.y * pan_speed;
        }

        // Scroll zoom
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.0 {
                self.camera.distance =
                    (self.camera.distance - scroll * 0.02 * self.camera_speed).clamp(1.5, 140.0);
            }
        }

        // 3. Draw 3D Ground Grid with radial distance attenuation
        if self.show_grid {
            self.draw_3d_grid(&painter, rect);
        }

        // 4. Draw Ground Contact Shadows under entities
        if self.show_ground_shadows {
            for ent in &scene.entities {
                if ent.kind != "light" && ent.kind != "camera" && ent.kind != "plane" && ent.visible
                {
                    let pos = ent.transform.position;
                    let shadow_center = self.project([pos[0], 0.01, pos[2]], rect);
                    if shadow_center.visible {
                        let rad = (ent.transform.scale[0] * 22.0 / shadow_center.depth.max(1.0))
                            .clamp(4.0, 80.0);
                        painter.circle_filled(
                            shadow_center.screen,
                            rad,
                            Color32::from_black_alpha(75),
                        );
                    }
                }
            }
        }

        // 5. Draw Scene Entities with Depth Sorting
        struct RenderEntity<'a> {
            entity: &'a Entity,
            depth: f32,
            center_screen: Pos2,
        }

        let eye = self.camera.eye_position();
        let mut render_list = Vec::new();

        for ent in &scene.entities {
            if !ent.visible {
                continue;
            }
            let pos = ent.transform.position;
            let dx = pos[0] - eye[0];
            let dy = pos[1] - eye[1];
            let dz = pos[2] - eye[2];
            let depth = (dx * dx + dy * dy + dz * dz).sqrt();
            let proj = self.project(pos, rect);

            render_list.push(RenderEntity {
                entity: ent,
                depth,
                center_screen: proj.screen,
            });
        }

        // Sort descending depth so further objects draw first
        render_list.sort_by(|a, b| {
            b.depth
                .partial_cmp(&a.depth)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for item in &render_list {
            let ent = item.entity;
            let is_selected = self.selected_id == Some(ent.id);
            self.draw_entity_3d(&painter, rect, ent, is_selected, accent);
        }

        // 6. Selection Raycast click
        if response.clicked_by(egui::PointerButton::Primary) {
            if let Some(pos) = response.interact_pointer_pos() {
                // Don't select if clicked the orientation compass in top-right
                let compass_rect = Rect::from_min_size(
                    rect.right_top() - Vec2::new(95.0, -10.0),
                    Vec2::new(85.0, 85.0),
                );
                if !compass_rect.contains(pos) {
                    let mut best_id = None;
                    let mut min_dist = 32.0;

                    for item in &render_list {
                        let dist = (item.center_screen - pos).length();
                        if dist < min_dist {
                            min_dist = dist;
                            best_id = Some(item.entity.id);
                        }
                    }
                    self.selected_id = best_id;
                }
            }
        }

        // 7. Draw 3D Transform Gizmo at Selected Entity
        if let Some(sel_id) = self.selected_id {
            if let Some(ent) = scene.entities.iter().find(|e| e.id == sel_id) {
                self.draw_transform_gizmo(&painter, rect, ent.transform.position);
            }
        }

        // 8. Draw 3D Orientation Compass (Top-Right)
        if self.show_orientation_gizmo {
            self.draw_orientation_compass(&painter, rect, &response);
        }

        // 9. Floating Top-Left Status Pill
        let (meshes, lights, cameras) = scene.stats();
        let status_rect = Rect::from_min_size(
            rect.left_top() + Vec2::new(12.0, 12.0),
            Vec2::new(280.0, 52.0),
        );
        painter.rect_filled(status_rect, 6.0, Color32::from_black_alpha(170));
        painter.rect_stroke(
            status_rect,
            6.0_f32,
            Stroke::new(1.0_f32, Color32::from_white_alpha(30)),
            egui::StrokeKind::Inside,
        );

        painter.text(
            status_rect.left_top() + Vec2::new(10.0, 8.0),
            egui::Align2::LEFT_TOP,
            format!("◈ CENA: {}  •  60 FPS", scene.name),
            egui::FontId::proportional(12.0),
            accent,
        );
        painter.text(
            status_rect.left_top() + Vec2::new(10.0, 26.0),
            egui::Align2::LEFT_TOP,
            format!(
                "Entidades: {} (Meshes: {}, Luzes: {}, Câmeras: {})\nDistância: {:.1}m  •  Câmera: {:.1}x",
                scene.entities.len(), meshes, lights, cameras, self.camera.distance, self.camera_speed
            ),
            egui::FontId::proportional(11.0),
            Color32::from_gray(180),
        );
    }

    fn draw_atmospheric_background(&self, painter: &egui::Painter, rect: Rect) {
        // Gradient background
        let top_color = Color32::from_rgb(13, 16, 24);
        let bot_color = Color32::from_rgb(22, 27, 40);

        painter.rect_filled(rect, 0.0, top_color);

        // Horizon subtle band
        let horizon_y = rect.center().y + (self.camera.pitch * 60.0);
        let horizon_rect =
            Rect::from_min_max(Pos2::new(rect.left(), horizon_y), rect.right_bottom());
        painter.rect_filled(horizon_rect, 0.0, bot_color);
    }

    fn draw_3d_grid(&self, painter: &egui::Painter, rect: Rect) {
        let grid_size: i32 = 12;
        let step = 1.0_f32;

        for i in -grid_size..=grid_size {
            let fi = i as f32 * step;
            let max_f = grid_size as f32 * step;

            let p1 = self.project([fi, 0.0, -max_f], rect);
            let p2 = self.project([fi, 0.0, max_f], rect);
            if p1.visible && p2.visible {
                let alpha = if i == 0 {
                    180
                } else {
                    (85 - (i.abs() * 5)).max(15) as u8
                };
                let col = if i == 0 {
                    Color32::from_rgba_unmultiplied(65, 125, 230, alpha)
                } else {
                    Color32::from_rgba_unmultiplied(75, 90, 115, alpha)
                };
                painter.line_segment(
                    [p1.screen, p2.screen],
                    Stroke::new(if i == 0 { 1.5_f32 } else { 0.6_f32 }, col),
                );
            }

            let q1 = self.project([-max_f, 0.0, fi], rect);
            let q2 = self.project([max_f, 0.0, fi], rect);
            if q1.visible && q2.visible {
                let alpha = if i == 0 {
                    180
                } else {
                    (85 - (i.abs() * 5)).max(15) as u8
                };
                let col = if i == 0 {
                    Color32::from_rgba_unmultiplied(230, 75, 75, alpha)
                } else {
                    Color32::from_rgba_unmultiplied(75, 90, 115, alpha)
                };
                painter.line_segment(
                    [q1.screen, q2.screen],
                    Stroke::new(if i == 0 { 1.5_f32 } else { 0.6_f32 }, col),
                );
            }
        }
    }

    fn draw_orientation_compass(
        &mut self,
        painter: &egui::Painter,
        rect: Rect,
        response: &egui::Response,
    ) {
        let center = rect.right_top() + Vec2::new(-45.0, 45.0);
        painter.circle_filled(center, 34.0, Color32::from_black_alpha(150));
        painter.circle_stroke(
            center,
            34.0,
            Stroke::new(1.0_f32, Color32::from_white_alpha(40)),
        );

        // Camera rotation angles
        let cos_p = self.camera.pitch.cos();
        let sin_p = self.camera.pitch.sin();
        let cos_y = self.camera.yaw.cos();
        let sin_y = self.camera.yaw.sin();

        let arm_len = 22.0_f32;

        // X axis projection in compass
        let x_screen = center + Vec2::new(cos_y * arm_len, sin_y * sin_p * arm_len);
        painter.line_segment(
            [center, x_screen],
            Stroke::new(2.5_f32, Color32::from_rgb(235, 65, 65)),
        );
        painter.circle_filled(x_screen, 4.5, Color32::from_rgb(235, 65, 65));
        painter.text(
            x_screen,
            egui::Align2::CENTER_CENTER,
            "X",
            egui::FontId::proportional(9.0),
            Color32::WHITE,
        );

        // Y axis (Up)
        let y_screen = center + Vec2::new(0.0, -cos_p * arm_len);
        painter.line_segment(
            [center, y_screen],
            Stroke::new(2.5_f32, Color32::from_rgb(65, 220, 85)),
        );
        painter.circle_filled(y_screen, 4.5, Color32::from_rgb(65, 220, 85));
        painter.text(
            y_screen,
            egui::Align2::CENTER_CENTER,
            "Y",
            egui::FontId::proportional(9.0),
            Color32::WHITE,
        );

        // Z axis (Forward/Depth)
        let z_screen = center + Vec2::new(-sin_y * arm_len, cos_y * sin_p * arm_len);
        painter.line_segment(
            [center, z_screen],
            Stroke::new(2.5_f32, Color32::from_rgb(65, 130, 245)),
        );
        painter.circle_filled(z_screen, 4.5, Color32::from_rgb(65, 130, 245));
        painter.text(
            z_screen,
            egui::Align2::CENTER_CENTER,
            "Z",
            egui::FontId::proportional(9.0),
            Color32::WHITE,
        );

        // Click to snap views
        if response.clicked_by(egui::PointerButton::Primary) {
            if let Some(pointer) = response.interact_pointer_pos() {
                if (pointer - x_screen).length() < 10.0 {
                    self.camera.snap_to_right();
                } else if (pointer - y_screen).length() < 10.0 {
                    self.camera.snap_to_top();
                } else if (pointer - z_screen).length() < 10.0 {
                    self.camera.snap_to_front();
                }
            }
        }
    }

    fn draw_entity_3d(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        ent: &Entity,
        is_selected: bool,
        accent: Color32,
    ) {
        let pos = ent.transform.position;
        let scale = ent.transform.scale;
        let base_c = ent.material.color;
        let fill_color = Color32::from_rgba_unmultiplied(
            (base_c[0] * 255.0) as u8,
            (base_c[1] * 255.0) as u8,
            (base_c[2] * 255.0) as u8,
            if self.wireframe_only { 30 } else { 190 },
        );
        let border_color = if is_selected {
            accent
        } else {
            Color32::from_rgba_unmultiplied(
                (base_c[0] * 210.0) as u8,
                (base_c[1] * 210.0) as u8,
                (base_c[2] * 210.0) as u8,
                255,
            )
        };

        match ent.kind.as_str() {
            "cube" | "plane" => {
                let hx = scale[0] * 0.5;
                let hy = if ent.kind == "plane" {
                    0.05
                } else {
                    scale[1] * 0.5
                };
                let hz = scale[2] * 0.5;

                // 8 corners of the 3D box
                let corners = [
                    [pos[0] - hx, pos[1] - hy, pos[2] - hz],
                    [pos[0] + hx, pos[1] - hy, pos[2] - hz],
                    [pos[0] + hx, pos[1] + hy, pos[2] - hz],
                    [pos[0] - hx, pos[1] + hy, pos[2] - hz],
                    [pos[0] - hx, pos[1] - hy, pos[2] + hz],
                    [pos[0] + hx, pos[1] - hy, pos[2] + hz],
                    [pos[0] + hx, pos[1] + hy, pos[2] + hz],
                    [pos[0] - hx, pos[1] + hy, pos[2] + hz],
                ];

                let mut proj = Vec::with_capacity(8);
                let mut all_visible = true;
                for c in &corners {
                    let p = self.project(*c, rect);
                    if !p.visible {
                        all_visible = false;
                    }
                    proj.push(p.screen);
                }

                if all_visible {
                    let faces = [
                        [0, 1, 2, 3], // Back
                        [4, 5, 6, 7], // Front
                        [0, 1, 5, 4], // Bottom
                        [2, 3, 7, 6], // Top
                        [0, 3, 7, 4], // Left
                        [1, 2, 6, 5], // Right
                    ];

                    for (f_idx, f) in faces.iter().enumerate() {
                        let pts = vec![proj[f[0]], proj[f[1]], proj[f[2]], proj[f[3]]];
                        let shade_mult = match f_idx {
                            3 => 1.25, // Top face brighter
                            1 => 1.0,  // Front face normal
                            5 => 0.9,  // Right face
                            4 => 0.75, // Left face
                            _ => 0.6,  // Back and bottom darker
                        };
                        let shaded_fill = Color32::from_rgba_unmultiplied(
                            ((fill_color.r() as f32 * shade_mult).min(255.0)) as u8,
                            ((fill_color.g() as f32 * shade_mult).min(255.0)) as u8,
                            ((fill_color.b() as f32 * shade_mult).min(255.0)) as u8,
                            fill_color.a(),
                        );
                        painter.add(egui::Shape::convex_polygon(
                            pts,
                            shaded_fill,
                            Stroke::new(if is_selected { 2.0_f32 } else { 1.0_f32 }, border_color),
                        ));
                    }
                }
            }

            // Imported GLB/GLTF assets are real mesh resources, not primitives.
            // This lightweight editor viewport cannot rasterize their triangles;
            // show an explicit asset marker instead of the misleading cube proxy.
            "model" => {
                let center = self.project(pos, rect);
                if center.visible {
                    let color = if is_selected {
                        accent
                    } else {
                        Color32::from_rgb(120, 205, 190)
                    };
                    painter.circle_filled(center.screen, 10.0, Color32::from_black_alpha(150));
                    painter.circle_stroke(center.screen, 10.0, Stroke::new(2.0_f32, color));
                    painter.text(
                        center.screen,
                        egui::Align2::CENTER_CENTER,
                        "GLB",
                        egui::FontId::proportional(8.0),
                        Color32::WHITE,
                    );
                }
            }

            "sphere" | "cylinder" | "capsule" | "cone" | "torus" | "monkey" => {
                let center = self.project(pos, rect);
                if center.visible {
                    let radius = (scale[0] * 35.0 / center.depth.max(1.0)).clamp(6.0, 140.0);

                    // Draw shaded sphere circle with depth rings
                    painter.circle_filled(center.screen, radius, fill_color);
                    painter.circle_stroke(
                        center.screen,
                        radius,
                        Stroke::new(if is_selected { 2.2_f32 } else { 1.2_f32 }, border_color),
                    );

                    painter.circle_stroke(
                        center.screen,
                        radius * 0.55,
                        Stroke::new(0.8_f32, Color32::from_white_alpha(70)),
                    );
                }
            }

            "light" => {
                let center = self.project(pos, rect);
                if center.visible {
                    let light_c = ent
                        .light
                        .as_ref()
                        .map(|l| l.color)
                        .unwrap_or([1.0, 0.9, 0.4]);
                    let col = Color32::from_rgb(
                        (light_c[0] * 255.0) as u8,
                        (light_c[1] * 255.0) as u8,
                        (light_c[2] * 255.0) as u8,
                    );
                    painter.circle_filled(center.screen, 8.0, col);
                    painter.circle_stroke(center.screen, 14.0, Stroke::new(1.5_f32, col));
                    for a in 0..8 {
                        let angle = (a as f32) * std::f32::consts::PI / 4.0;
                        let r1 = 16.0;
                        let r2 = 22.0;
                        let p1 = center.screen + Vec2::new(angle.cos() * r1, angle.sin() * r1);
                        let p2 = center.screen + Vec2::new(angle.cos() * r2, angle.sin() * r2);
                        painter.line_segment([p1, p2], Stroke::new(1.2_f32, col));
                    }
                }
            }

            "camera" => {
                let center = self.project(pos, rect);
                if center.visible {
                    let col = Color32::from_rgb(90, 180, 255);
                    painter.circle_filled(center.screen, 7.0, col);
                    let forward_p = self.project([pos[0], pos[1] - 0.5, pos[2] - 3.0], rect);
                    if forward_p.visible {
                        painter.line_segment(
                            [center.screen, forward_p.screen],
                            Stroke::new(1.5_f32, col),
                        );
                        painter.rect_stroke(
                            Rect::from_center_size(forward_p.screen, Vec2::new(32.0, 20.0)),
                            2.0,
                            Stroke::new(1.2_f32, col),
                            egui::StrokeKind::Middle,
                        );
                    }
                }
            }

            _ => {
                let center = self.project(pos, rect);
                if center.visible {
                    painter.circle_filled(center.screen, 6.0, border_color);
                }
            }
        }

        // Entity name tag with rounded background badge
        let center = self.project([pos[0], pos[1] + scale[1] * 0.6 + 0.35, pos[2]], rect);
        if center.visible {
            let label_col = if is_selected {
                accent
            } else {
                Color32::from_gray(210)
            };
            painter.text(
                center.screen,
                egui::Align2::CENTER_BOTTOM,
                &ent.name,
                egui::FontId::proportional(11.0),
                label_col,
            );
        }
    }

    fn draw_transform_gizmo(&self, painter: &egui::Painter, rect: Rect, pos: [f32; 3]) {
        let center = self.project(pos, rect);
        if !center.visible {
            return;
        }

        let axis_len = 1.7;
        let px = self.project([pos[0] + axis_len, pos[1], pos[2]], rect);
        let py = self.project([pos[0], pos[1] + axis_len, pos[2]], rect);
        let pz = self.project([pos[0], pos[1], pos[2] + axis_len], rect);

        match self.transform_mode {
            TransformToolMode::Translate => {
                // X Axis (Red)
                if px.visible {
                    painter.line_segment(
                        [center.screen, px.screen],
                        Stroke::new(2.5_f32, Color32::from_rgb(240, 60, 60)),
                    );
                    painter.circle_filled(px.screen, 4.5, Color32::from_rgb(240, 60, 60));
                }
                // Y Axis (Green)
                if py.visible {
                    painter.line_segment(
                        [center.screen, py.screen],
                        Stroke::new(2.5_f32, Color32::from_rgb(60, 220, 80)),
                    );
                    painter.circle_filled(py.screen, 4.5, Color32::from_rgb(60, 220, 80));
                }
                // Z Axis (Blue)
                if pz.visible {
                    painter.line_segment(
                        [center.screen, pz.screen],
                        Stroke::new(2.5_f32, Color32::from_rgb(60, 130, 255)),
                    );
                    painter.circle_filled(pz.screen, 4.5, Color32::from_rgb(60, 130, 255));
                }
            }
            TransformToolMode::Rotate => {
                // Draw 3 rotational rings around center
                painter.circle_stroke(
                    center.screen,
                    32.0,
                    Stroke::new(1.8_f32, Color32::from_rgb(240, 60, 60)),
                );
                painter.circle_stroke(
                    center.screen,
                    24.0,
                    Stroke::new(1.8_f32, Color32::from_rgb(60, 220, 80)),
                );
                painter.circle_stroke(
                    center.screen,
                    16.0,
                    Stroke::new(1.8_f32, Color32::from_rgb(60, 130, 255)),
                );
            }
            TransformToolMode::Scale => {
                // Draw cube-ended scale axes
                if px.visible {
                    painter.line_segment(
                        [center.screen, px.screen],
                        Stroke::new(2.5_f32, Color32::from_rgb(240, 60, 60)),
                    );
                    painter.rect_filled(
                        Rect::from_center_size(px.screen, Vec2::new(7.0, 7.0)),
                        1.0,
                        Color32::from_rgb(240, 60, 60),
                    );
                }
                if py.visible {
                    painter.line_segment(
                        [center.screen, py.screen],
                        Stroke::new(2.5_f32, Color32::from_rgb(60, 220, 80)),
                    );
                    painter.rect_filled(
                        Rect::from_center_size(py.screen, Vec2::new(7.0, 7.0)),
                        1.0,
                        Color32::from_rgb(60, 220, 80),
                    );
                }
                if pz.visible {
                    painter.line_segment(
                        [center.screen, pz.screen],
                        Stroke::new(2.5_f32, Color32::from_rgb(60, 130, 255)),
                    );
                    painter.rect_filled(
                        Rect::from_center_size(pz.screen, Vec2::new(7.0, 7.0)),
                        1.0,
                        Color32::from_rgb(60, 130, 255),
                    );
                }
            }
        }
    }
}
