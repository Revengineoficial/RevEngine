use crate::scene::{Entity, PhysicsData, Scene};

pub struct ProceduralSceneGenerator;

impl ProceduralSceneGenerator {
    /// Generates a rich, fully populated Medieval Fortress & Castle scene with
    /// extensive terrain, fortified stone walls, watchtowers, glowing perimeter torches,
    /// keep, gatehouse, cinematic lighting and physics.
    pub fn create_medieval_castle() -> Scene {
        let mut scene = Scene::new_empty("Cenário Medieval - Fortaleza & Castelo");

        // 1. Extensivo Terreno com Física
        let terrain_id = scene.alloc_id();
        let mut terrain = Entity::new_plane(
            terrain_id,
            "Terreno_Extensivo",
            [0.0, -0.05, 0.0],
            65.0,
            [0.18, 0.26, 0.15, 1.0], // Grama úmida / terra medieval
        );
        terrain.material.roughness = 0.95;
        terrain.material.metallic = 0.02;
        terrain.physics = PhysicsData {
            enabled: true,
            body_type: "Static".into(),
            collider: "box".into(),
            friction: 0.8,
            restitution: 0.05,
            mass: 0.0,
        };
        scene.add_entity(terrain);

        // 2. Castelo Central (Keep / Fortaleza Principal)
        let keep_id = scene.alloc_id();
        let mut keep = Entity::new_cube(
            keep_id,
            "Castelo_Fortaleza_Central",
            [0.0, 4.0, 0.0],
            1.0,
            [0.45, 0.46, 0.48, 1.0], // Pedra cinzelada
        );
        keep.transform.scale = [10.0, 8.0, 10.0];
        keep.material.roughness = 0.85;
        keep.material.metallic = 0.1;
        keep.physics = PhysicsData {
            enabled: true,
            body_type: "Static".into(),
            collider: "box".into(),
            ..Default::default()
        };
        scene.add_entity(keep);

        // Torreão Superior do Castelo
        let tower_keep_id = scene.alloc_id();
        let mut tower_keep = Entity::new_cube(
            tower_keep_id,
            "Torreao_Superior_Castelo",
            [0.0, 9.5, 0.0],
            1.0,
            [0.38, 0.39, 0.41, 1.0],
        );
        tower_keep.transform.scale = [6.0, 4.0, 6.0];
        scene.add_entity(tower_keep);

        // Telhado Real
        let roof_id = scene.alloc_id();
        let mut roof = Entity::new_cube(
            roof_id,
            "Telhado_Cume_Real",
            [0.0, 12.5, 0.0],
            1.0,
            [0.65, 0.16, 0.16, 1.0], // Telhas vermelhas nobres
        );
        roof.transform.scale = [4.5, 2.2, 4.5];
        scene.add_entity(roof);

        // 3. Quatro Torres de Vigia Fortificadas nos Cantos
        let tower_coords = [
            ("Torre_Noroeste", [-14.0, 5.5, -14.0]),
            ("Torre_Nordeste", [14.0, 5.5, -14.0]),
            ("Torre_Sudoeste", [-14.0, 5.5, 14.0]),
            ("Torre_Sudeste", [14.0, 5.5, 14.0]),
        ];

        for (name, pos) in tower_coords {
            let tid = scene.alloc_id();
            let mut tower = Entity::new_cube(tid, name, pos, 1.0, [0.42, 0.43, 0.46, 1.0]);
            tower.transform.scale = [4.0, 11.0, 4.0];
            tower.material.roughness = 0.8;
            tower.physics = PhysicsData {
                enabled: true,
                body_type: "Static".into(),
                collider: "box".into(),
                ..Default::default()
            };
            scene.add_entity(tower);

            // Ameia de topo da torre
            let battlement_id = scene.alloc_id();
            let mut battlement = Entity::new_cube(
                battlement_id,
                &format!("Ameia_{name}"),
                [pos[0], pos[1] + 6.0, pos[2]],
                1.0,
                [0.35, 0.36, 0.38, 1.0],
            );
            battlement.transform.scale = [4.6, 1.2, 4.6];
            scene.add_entity(battlement);
        }

        // 4. Muralhas de Pedra Conectando as Torres
        // Muralha Norte
        let wn_id = scene.alloc_id();
        let mut wn = Entity::new_cube(wn_id, "Muralha_Norte", [0.0, 3.2, -14.0], 1.0, [0.48, 0.49, 0.52, 1.0]);
        wn.transform.scale = [24.0, 6.4, 2.2];
        scene.add_entity(wn);

        // Muralha Leste
        let we_id = scene.alloc_id();
        let mut we = Entity::new_cube(we_id, "Muralha_Leste", [14.0, 3.2, 0.0], 1.0, [0.48, 0.49, 0.52, 1.0]);
        we.transform.scale = [2.2, 6.4, 24.0];
        scene.add_entity(we);

        // Muralha Oeste
        let ww_id = scene.alloc_id();
        let mut ww = Entity::new_cube(ww_id, "Muralha_Oeste", [-14.0, 3.2, 0.0], 1.0, [0.48, 0.49, 0.52, 1.0]);
        ww.transform.scale = [2.2, 6.4, 24.0];
        scene.add_entity(ww);

        // Muralha Sul (Dividida para o Portão Principal)
        let ws1_id = scene.alloc_id();
        let mut ws1 = Entity::new_cube(ws1_id, "Muralha_Sul_Esquerda", [-8.0, 3.2, 14.0], 1.0, [0.48, 0.49, 0.52, 1.0]);
        ws1.transform.scale = [9.0, 6.4, 2.2];
        scene.add_entity(ws1);

        let ws2_id = scene.alloc_id();
        let mut ws2 = Entity::new_cube(ws2_id, "Muralha_Sul_Direita", [8.0, 3.2, 14.0], 1.0, [0.48, 0.49, 0.52, 1.0]);
        ws2.transform.scale = [9.0, 6.4, 2.2];
        scene.add_entity(ws2);

        // Portão Levadiço de Madeira de Carvalho
        let gate_id = scene.alloc_id();
        let mut gate = Entity::new_cube(gate_id, "Portao_Levadico_Madeira", [0.0, 2.2, 14.0], 1.0, [0.38, 0.22, 0.12, 1.0]);
        gate.transform.scale = [4.8, 4.4, 0.8];
        gate.material.roughness = 0.75;
        scene.add_entity(gate);

        // Arco do Portão
        let arch_id = scene.alloc_id();
        let mut arch = Entity::new_cube(arch_id, "Arco_Pedra_Portao", [0.0, 5.0, 14.0], 1.0, [0.35, 0.36, 0.39, 1.0]);
        arch.transform.scale = [6.5, 1.5, 2.4];
        scene.add_entity(arch);

        // 5. Tochas Iluminando o Perímetro (Postes de Madeira com Chamas Quentes)
        let torch_positions = [
            ("Tocha_Portao_Esquerdo", [-3.2, 2.2, 14.8], [-3.2, 3.6, 15.0]),
            ("Tocha_Portao_Direito", [3.2, 2.2, 14.8], [3.2, 3.6, 15.0]),
            ("Tocha_Muralha_Norte_1", [-6.0, 4.2, -13.0], [-6.0, 5.2, -13.0]),
            ("Tocha_Muralha_Norte_2", [6.0, 4.2, -13.0], [6.0, 5.2, -13.0]),
            ("Tocha_Torre_Noroeste", [-14.0, 11.5, -14.0], [-14.0, 12.5, -14.0]),
            ("Tocha_Torre_Nordeste", [14.0, 11.5, -14.0], [14.0, 12.5, -14.0]),
            ("Tocha_Torre_Sudoeste", [-14.0, 11.5, 14.0], [-14.0, 12.5, 14.0]),
            ("Tocha_Torre_Sudeste", [14.0, 11.5, 14.0], [14.0, 12.5, 14.0]),
        ];

        for (name, stand_pos, flame_pos) in torch_positions {
            // Suporte de tocha
            let stand_id = scene.alloc_id();
            let mut stand = Entity::new_cube(stand_id, &format!("{name}_Suporte"), stand_pos, 1.0, [0.25, 0.16, 0.10, 1.0]);
            stand.transform.scale = [0.3, 1.6, 0.3];
            scene.add_entity(stand);

            // Chama brilhante (mesh emissivo)
            let flame_id = scene.alloc_id();
            let mut flame = Entity::new_cube(flame_id, &format!("{name}_Chama"), flame_pos, 1.0, [1.0, 0.55, 0.1, 1.0]);
            flame.transform.scale = [0.4, 0.6, 0.4];
            flame.material.emissive = [1.0, 0.55, 0.1];
            scene.add_entity(flame);

            // Luz pontual real
            let light_id = scene.alloc_id();
            let mut light_ent = Entity::new_light(
                light_id,
                &format!("Luz_{name}"),
                flame_pos,
                "point",
                [1.0, 0.62, 0.22],
                3200.0,
            );
            if let Some(l) = light_ent.light.as_mut() {
                l.range = 22.0;
            }
            scene.add_entity(light_ent);
        }

        // 6. Iluminação Global Medieval (Sol dourado de crepúsculo)
        let sun_id = scene.alloc_id();
        let sun = Entity::new_light(
            sun_id,
            "Sol_Medieval_Crepusculo",
            [25.0, 32.0, 20.0],
            "directional",
            [1.0, 0.88, 0.68],
            14000.0,
        );
        scene.add_entity(sun);

        // 7. Câmera Cinemática focando na Fortaleza
        let cam_id = scene.alloc_id();
        let mut cam = Entity::new_camera(cam_id, "Camera_Cinematica_Fortaleza", [26.0, 15.0, 30.0]);
        cam.transform.rotation = [-18.0, 42.0, 0.0];
        scene.add_entity(cam);

        scene
    }

    /// Generates a Sci-Fi Space Racing track with neon waypoints, obstacles, spaceship and cosmic lighting
    pub fn create_space_racing() -> Scene {
        let mut scene = Scene::new_empty("Cenário Sci-Fi - Corrida Espacial");

        // Pista Principal de Neon
        let track_id = scene.alloc_id();
        let mut track = Entity::new_plane(track_id, "Pista_Interestelar", [0.0, 0.0, 0.0], 80.0, [0.05, 0.07, 0.15, 1.0]);
        track.material.metallic = 0.85;
        track.material.roughness = 0.2;
        scene.add_entity(track);

        // Nave do Jogador
        let ship_id = scene.alloc_id();
        let mut ship = Entity::new_cube(ship_id, "Nave_Caca_Estelar", [0.0, 1.2, 0.0], 1.0, [0.15, 0.65, 1.0, 1.0]);
        ship.transform.scale = [2.2, 0.8, 3.8];
        ship.material.metallic = 0.9;
        ship.material.roughness = 0.15;
        ship.physics = PhysicsData {
            enabled: true,
            body_type: "Dynamic".into(),
            collider: "box".into(),
            mass: 1200.0,
            ..Default::default()
        };
        scene.add_entity(ship);

        // Checkpoints de Neon
        for i in 1..=4 {
            let cp_id = scene.alloc_id();
            let z_pos = (i as f32) * 18.0;
            let mut cp = Entity::new_cube(cp_id, &format!("Portal_Neon_{i}"), [0.0, 4.0, z_pos], 1.0, [0.95, 0.1, 0.8, 1.0]);
            cp.transform.scale = [12.0, 7.0, 1.0];
            cp.material.emissive = [0.95, 0.1, 0.8];
            scene.add_entity(cp);
        }

        // Luz Cósmica
        let star_id = scene.alloc_id();
        let star = Entity::new_light(star_id, "Estrela_Nebulosa", [10.0, 30.0, -20.0], "directional", [0.6, 0.8, 1.0], 16000.0);
        scene.add_entity(star);

        let cam_id = scene.alloc_id();
        let cam = Entity::new_camera(cam_id, "Camera_Corrida", [0.0, 5.0, -12.0]);
        scene.add_entity(cam);

        scene
    }

    /// Generates a Fantasy Forest scene with trees, firecamp and elven ruins
    pub fn create_fantasy_forest() -> Scene {
        let mut scene = Scene::new_empty("Cenário Fantasia - Floresta Élfica");

        let ground_id = scene.alloc_id();
        let ground = Entity::new_plane(ground_id, "Solo_Floresta", [0.0, 0.0, 0.0], 60.0, [0.12, 0.22, 0.1, 1.0]);
        scene.add_entity(ground);

        // Árvores com troncos e copas esmeralda
        let tree_coords = [
            [-8.0, 0.0, -6.0],
            [10.0, 0.0, -8.0],
            [-12.0, 0.0, 8.0],
            [12.0, 0.0, 12.0],
            [-4.0, 0.0, 14.0],
            [6.0, 0.0, -14.0],
        ];

        for (idx, pos) in tree_coords.iter().enumerate() {
            let trunk_id = scene.alloc_id();
            let mut trunk = Entity::new_cube(trunk_id, &format!("Tronco_{}", idx + 1), [pos[0], 2.5, pos[2]], 1.0, [0.35, 0.22, 0.12, 1.0]);
            trunk.transform.scale = [0.9, 5.0, 0.9];
            scene.add_entity(trunk);

            let leaf_id = scene.alloc_id();
            let mut leaf = Entity::new_cube(leaf_id, &format!("Copa_{}", idx + 1), [pos[0], 5.8, pos[2]], 1.0, [0.15, 0.55, 0.2, 1.0]);
            leaf.transform.scale = [3.8, 3.5, 3.8];
            scene.add_entity(leaf);
        }

        // Fogueira no centro
        let fire_id = scene.alloc_id();
        let mut fire = Entity::new_cube(fire_id, "Fogueira_Acampamento", [0.0, 0.4, 0.0], 1.0, [1.0, 0.5, 0.1, 1.0]);
        fire.transform.scale = [1.0, 0.8, 1.0];
        fire.material.emissive = [1.0, 0.5, 0.1];
        scene.add_entity(fire);

        let fire_light_id = scene.alloc_id();
        let fire_light = Entity::new_light(fire_light_id, "Luz_Fogueira", [0.0, 1.5, 0.0], "point", [1.0, 0.6, 0.15], 4000.0);
        scene.add_entity(fire_light);

        let sun_id = scene.alloc_id();
        let sun = Entity::new_light(sun_id, "Luz_Lunar_Suave", [5.0, 25.0, 15.0], "directional", [0.55, 0.75, 1.0], 6000.0);
        scene.add_entity(sun);

        let cam_id = scene.alloc_id();
        let cam = Entity::new_camera(cam_id, "Camera_Floresta", [12.0, 8.0, 16.0]);
        scene.add_entity(cam);

        scene
    }

    /// Generates a Complete Roblox Obby / Parkour Game with Spawn, Floating Obstacles,
    /// Lava Killparts, Checkpoints, Trampoline Bounce Pad, and Winner Gold Trophy Island
    pub fn create_roblox_obby_game() -> Scene {
        let mut scene = Scene::new_empty("Roblox Obby - Aventura no Espaço");

        // 1. Plataforma de Spawn do Roblox
        let spawn_id = scene.alloc_id();
        let mut spawn = Entity::new_cube(
            spawn_id,
            "SpawnLocation_Principal",
            [0.0, 0.0, 0.0],
            1.0,
            [0.15, 0.55, 0.95, 1.0], // Azul clássico Roblox
        );
        spawn.transform.scale = [14.0, 1.2, 14.0];
        spawn.material.metallic = 0.2;
        spawn.material.roughness = 0.3;
        spawn.physics = PhysicsData {
            enabled: true,
            body_type: "Static".into(),
            collider: "box".into(),
            ..Default::default()
        };
        scene.add_entity(spawn);

        // Borda de Neon da plataforma de Spawn
        let border_id = scene.alloc_id();
        let mut border = Entity::new_cube(border_id, "Neon_Border_Spawn", [0.0, 0.7, 0.0], 1.0, [0.2, 0.85, 1.0, 1.0]);
        border.transform.scale = [14.4, 0.3, 14.4];
        border.material.emissive = [0.2, 0.85, 1.0];
        scene.add_entity(border);

        // 2. Obstáculos Flutuantes (Parkour Pads com cores Neon)
        let parkour_steps = [
            ("Parkour_Pad_1_Ciano", [0.0, 1.2, 13.0], [5.0, 0.8, 5.0], [0.1, 0.9, 0.9, 1.0]),
            ("Parkour_Pad_2_Amarelo", [3.5, 2.6, 21.0], [4.5, 0.8, 4.5], [0.95, 0.85, 0.1, 1.0]),
            ("Parkour_Pad_3_Magenta", [-3.5, 4.0, 29.0], [4.5, 0.8, 4.5], [0.95, 0.15, 0.75, 1.0]),
            ("Parkour_Pad_4_Verde", [0.0, 5.5, 37.0], [4.2, 0.8, 4.2], [0.2, 0.95, 0.3, 1.0]),
        ];

        for (name, pos, scale, color) in parkour_steps {
            let pid = scene.alloc_id();
            let mut p_ent = Entity::new_cube(pid, name, pos, 1.0, color);
            p_ent.transform.scale = scale;
            p_ent.material.emissive = [color[0] * 0.4, color[1] * 0.4, color[2] * 0.4];
            p_ent.physics = PhysicsData {
                enabled: true,
                body_type: "Static".into(),
                collider: "box".into(),
                ..Default::default()
            };
            scene.add_entity(p_ent);
        }

        // 3. Ilha de Checkpoint 1
        let cp1_id = scene.alloc_id();
        let mut cp1 = Entity::new_cube(cp1_id, "Checkpoint_Island_1", [0.0, 6.8, 48.0], 1.0, [0.2, 0.85, 0.4, 1.0]);
        cp1.transform.scale = [10.0, 1.0, 10.0];
        scene.add_entity(cp1);

        // Mastro e Bandeira de Checkpoint
        let pole_id = scene.alloc_id();
        let mut pole = Entity::new_cube(pole_id, "Checkpoint_Flagpole", [0.0, 10.0, 48.0], 1.0, [0.9, 0.9, 0.9, 1.0]);
        pole.transform.scale = [0.4, 6.0, 0.4];
        scene.add_entity(pole);

        // 4. Zona de Perigo: Lava Killpart (Glow Neon Vermelho)
        let lava_id = scene.alloc_id();
        let mut lava = Entity::new_cube(lava_id, "Lava_KillPart_Perigo", [0.0, 6.0, 70.0], 1.0, [1.0, 0.12, 0.05, 1.0]);
        lava.transform.scale = [32.0, 0.8, 28.0];
        lava.material.emissive = [1.0, 0.2, 0.05];
        scene.add_entity(lava);

        // Luz vermelha emissiva da Lava
        let lava_light_id = scene.alloc_id();
        let lava_light = Entity::new_light(lava_light_id, "Luz_Neon_Lava", [0.0, 8.5, 70.0], "point", [1.0, 0.2, 0.05], 5000.0);
        scene.add_entity(lava_light);

        // Passarelas estreitas sobre a Lava
        let lava_steps = [
            ("Passo_Lava_1", [-2.5, 8.2, 62.0], [2.2, 0.6, 2.2]),
            ("Passo_Lava_2", [2.5, 9.4, 70.0], [2.2, 0.6, 2.2]),
            ("Passo_Lava_3", [-2.0, 10.6, 78.0], [2.2, 0.6, 2.2]),
        ];
        for (name, pos, scale) in lava_steps {
            let sid = scene.alloc_id();
            let mut step_ent = Entity::new_cube(sid, name, pos, 1.0, [0.35, 0.38, 0.45, 1.0]);
            step_ent.transform.scale = scale;
            scene.add_entity(step_ent);
        }

        // 5. Trampolim / Super Jump Pad (Amarelo Neon)
        let jump_id = scene.alloc_id();
        let mut jump_pad = Entity::new_cube(jump_id, "Trampolim_JumpPad_Super", [0.0, 11.8, 88.0], 1.0, [1.0, 0.9, 0.1, 1.0]);
        jump_pad.transform.scale = [6.0, 0.8, 6.0];
        jump_pad.material.emissive = [1.0, 0.85, 0.1];
        jump_pad.physics = PhysicsData {
            enabled: true,
            body_type: "Static".into(),
            collider: "box".into(),
            restitution: 0.98, // Super quique!
            friction: 0.2,
            ..Default::default()
        };
        scene.add_entity(jump_pad);

        // 6. Ilha Final de Vitória e Troféu Dourado
        let win_id = scene.alloc_id();
        let mut win_island = Entity::new_cube(win_id, "Ilha_Vitoria_Campeao", [0.0, 22.0, 112.0], 1.0, [0.45, 0.18, 0.75, 1.0]);
        win_island.transform.scale = [18.0, 1.4, 18.0];
        scene.add_entity(win_island);

        // Troféu Dourado
        let trophy_id = scene.alloc_id();
        let mut trophy = Entity::new_cube(trophy_id, "Trofeu_Dourado_Vencedor", [0.0, 25.5, 112.0], 1.0, [1.0, 0.84, 0.0, 1.0]);
        trophy.transform.scale = [3.2, 4.5, 3.2];
        trophy.material.metallic = 0.95;
        trophy.material.roughness = 0.1;
        trophy.material.emissive = [0.8, 0.65, 0.1];
        scene.add_entity(trophy);

        // Holofotes de Vitória
        let win_light_id = scene.alloc_id();
        let win_light = Entity::new_light(win_light_id, "Holofote_Vitoria", [0.0, 32.0, 112.0], "point", [1.0, 0.9, 0.4], 8000.0);
        scene.add_entity(win_light);

        // Sol e Câmera
        let sun_id = scene.alloc_id();
        let sun = Entity::new_light(sun_id, "Sol_Roblox_Daylight", [30.0, 50.0, 40.0], "directional", [1.0, 0.98, 0.92], 16000.0);
        scene.add_entity(sun);

        let cam_id = scene.alloc_id();
        let mut cam = Entity::new_camera(cam_id, "Camera_Roblox_Cinematica", [16.0, 12.0, -10.0]);
        cam.transform.rotation = [-15.0, 32.0, 0.0];
        scene.add_entity(cam);

        scene
    }

    /// Evaluates user prompt and semantic cues to synthesize a rich 3D scene
    pub fn match_or_generate_scene(prompt: &str, raw_reply: &str) -> Option<Scene> {
        let combined = format!("{prompt} {raw_reply}").to_lowercase();

        // 1. Roblox Games / Luau
        if combined.contains("roblox")
            || combined.contains("obby")
            || combined.contains("parkour")
            || combined.contains("luau")
            || combined.contains("rbxlx")
        {
            return Some(Self::create_roblox_obby_game());
        }

        if combined.contains("medieval")
            || combined.contains("castelo")
            || combined.contains("muralha")
            || combined.contains("tocha")
            || combined.contains("torre de vigia")
            || combined.contains("fortaleza")
        {
            return Some(Self::create_medieval_castle());
        }

        if combined.contains("corrida")
            || combined.contains("espacial")
            || combined.contains("espaço")
            || combined.contains("asteroide")
            || combined.contains("nave")
            || combined.contains("sci-fi")
        {
            return Some(Self::create_space_racing());
        }

        if combined.contains("elfo")
            || combined.contains("fantasia")
            || combined.contains("floresta")
            || combined.contains("arvore")
            || combined.contains("árvore")
            || combined.contains("natureza")
        {
            return Some(Self::create_fantasy_forest());
        }

        None
    }
}
