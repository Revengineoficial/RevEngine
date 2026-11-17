use crate::scene::{Entity, Scene};
use std::{fs, path::Path};

pub struct RobloxBridge;

impl RobloxBridge {
    /// Export the RevEngine scene to a native Roblox Studio .rbxlx place file
    /// along with a modern Rojo project configuration and structured Luau scripts.
    pub fn export_project(scene: &Scene, target_dir: &Path) -> Result<String, String> {
        let roblox_dir = target_dir.join("Integrations").join("Roblox");
        let src_server = roblox_dir.join("src").join("server");
        let src_client = roblox_dir.join("src").join("client");
        let src_shared = roblox_dir.join("src").join("shared");
        let assets_dir = roblox_dir.join("assets");

        fs::create_dir_all(&src_server).map_err(|e| e.to_string())?;
        fs::create_dir_all(&src_client).map_err(|e| e.to_string())?;
        fs::create_dir_all(&src_shared).map_err(|e| e.to_string())?;
        fs::create_dir_all(&assets_dir).map_err(|e| e.to_string())?;

        // 1. Generate default.project.json (Rojo standard)
        let rojo_project = format!(
            r#"{{
  "name": "{}",
  "tree": {{
    "$className": "DataModel",
    "ServerScriptService": {{
      "$path": "src/server"
    }},
    "ReplicatedStorage": {{
      "$path": "src/shared"
    }},
    "StarterPlayer": {{
      "StarterPlayerScripts": {{
        "$path": "src/client"
      }}
    }}
  }}
}}
"#,
            scene.name.replace(' ', "_")
        );
        fs::write(roblox_dir.join("default.project.json"), rojo_project)
            .map_err(|e| e.to_string())?;

        // 2. Generate ServerScriptService Luau Game Manager
        let server_script = format!(
            r#"--!strict
-- GameManager.server.luau
-- Generated automatically by RevEngine Studio 0.2.0 for Roblox Studio
-- Game: {}

local Players = game:GetService("Players")
local Workspace = game:GetService("Workspace")
local ReplicatedStorage = game:GetService("ReplicatedStorage")

print("◈ [RevEngine] Servidor Roblox inicializado com sucesso para: {}")

-- Leaderstats (Moedas, Nível e Pontuação)
local function onPlayerAdded(player: Player)
	local leaderstats = Instance.new("Folder")
	leaderstats.Name = "leaderstats"
	leaderstats.Parent = player

	local points = Instance.new("IntValue")
	points.Name = "Pontos"
	points.Value = 0
	points.Parent = leaderstats

	local level = Instance.new("IntValue")
	level.Name = "Nivel"
	level.Value = 1
	level.Parent = leaderstats

	player.CharacterAdded:Connect(function(character)
		local humanoid = character:WaitForChild("Humanoid") :: Humanoid
		humanoid.Died:Connect(function()
			print(player.Name .. " reiniciou a fase.")
		end)
	end)
end

Players.PlayerAdded:Connect(onPlayerAdded)

-- Detectar blocos de Lava / Killparts gerados pelo RevEngine
for _, part in ipairs(Workspace:GetDescendants()) do
	if part:IsA("BasePart") then
		local nameLower = string.lower(part.Name)
		if string.find(nameLower, "lava") or string.find(nameLower, "kill") or string.find(nameLower, "morte") then
			part.Touched:Connect(function(hit)
				local character = hit.Parent
				local humanoid = character and character:FindFirstChildOfClass("Humanoid")
				if humanoid and humanoid.Health > 0 then
					humanoid.Health = 0
				end
			end)
		elseif string.find(nameLower, "jump") or string.find(nameLower, "trampolim") then
			part.Touched:Connect(function(hit)
				local root = hit.Parent and hit.Parent:FindFirstChild("HumanoidRootPart") :: BasePart
				if root then
					root.AssemblyLinearVelocity = Vector3.new(0, 95, 0)
				end
			end)
		elseif string.find(nameLower, "moeda") or string.find(nameLower, "coin") then
			part.Touched:Connect(function(hit)
				local player = Players:GetPlayerFromCharacter(hit.Parent)
				if player then
					local leaderstats = player:FindFirstChild("leaderstats")
					local pts = leaderstats and leaderstats:FindFirstChild("Pontos") :: IntValue
					if pts then
						pts.Value += 10
					end
					part:Destroy()
				end
			end)
		end
	end
end
"#,
            scene.name, scene.name
        );
        fs::write(src_server.join("GameManager.server.luau"), server_script)
            .map_err(|e| e.to_string())?;

        // 3. Generate StarterPlayerScripts Client Controller
        let client_script = format!(
            r#"--!strict
-- CameraController.client.luau
-- Generated automatically by RevEngine Studio 0.2.0
-- Game: {}

local Players = game:GetService("Players")
local RunService = game:GetService("RunService")
local TweenService = game:GetService("TweenService")
local Workspace = game:GetService("Workspace")

local player = Players.LocalPlayer
local camera = Workspace.CurrentCamera

print("◈ [RevEngine] Controlador de cliente Luau ativo para: " .. player.Name)

-- Configuração cinemática e feedback de interface
local function setupAtmosphere()
	local lighting = game:GetService("Lighting")
	lighting.ClockTime = 14.5
	lighting.Brightness = 2.4
	lighting.OutdoorAmbient = Color3.fromRGB(130, 140, 160)
end

setupAtmosphere()
"#,
            scene.name
        );
        fs::write(src_client.join("CameraController.client.luau"), client_script)
            .map_err(|e| e.to_string())?;

        // 4. Generate ReplicatedStorage GameConfig Module
        let shared_config = format!(
            r#"--!strict
-- GameConfig.luau
-- Configurações globais partilhadas entre Servidor e Cliente

local GameConfig = {{
	GameTitle = "{}",
	Engine = "RevEngine Studio 0.2.0 -> Roblox Studio Native",
	Gravity = 196.2,
	WalkSpeed = 20,
	JumpPower = 52,
	ObjectsCount = {},
}}

return GameConfig
"#,
            scene.name,
            scene.entities.len()
        );
        fs::write(src_shared.join("GameConfig.luau"), shared_config)
            .map_err(|e| e.to_string())?;

        // 5. Generate Official Roblox XML Place File (.rbxlx)
        // This is directly double-clickable and openable in Roblox Studio!
        let rbxlx_path = roblox_dir.join(format!("{}.rbxlx", scene.name.replace(' ', "_")));
        let rbxlx_content = Self::generate_rbxlx_xml(scene);
        fs::write(&rbxlx_path, rbxlx_content).map_err(|e| e.to_string())?;

        Ok(rbxlx_path.to_string_lossy().to_string())
    }

    /// Generates standard Roblox Place XML (.rbxlx) containing all scene parts,
    /// positions, sizes, colors, anchored physics, lights, and scripts.
    pub fn generate_rbxlx_xml(scene: &Scene) -> String {
        let mut xml = String::new();
        xml.push_str(r#"<roblox xmlns:xmime="http://www.w3.org/2005/05/xmlmime" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="http://www.roblox.com/roblox.xsd" version="4">
	<External>null</External>
	<External>nil</External>
	<Item class="Workspace" referent="RBX_WORKSPACE">
		<Properties>
			<string name="Name">Workspace</string>
			<bool name="FilteringEnabled">true</bool>
		</Properties>
"#);

        let mut ref_id: u64 = 1000;

        for ent in &scene.entities {
            if ent.kind == "camera" {
                continue;
            }

            ref_id += 1;
            let p = ent.transform.position;
            let s = ent.transform.scale;
            let c = ent.material.color;
            let is_anchored = if ent.physics.enabled {
                ent.physics.body_type == "Static"
            } else {
                true
            };

            // Shape token: 1 = Block, 0 = Ball, 2 = Cylinder
            let shape_token = match ent.kind.as_str() {
                "sphere" => 0,
                "cylinder" => 2,
                _ => 1,
            };

            // Height and scale adjustments
            let (sx, sy, sz) = if ent.kind == "plane" {
                (s[0], 0.2, s[2])
            } else {
                (s[0], s[1], s[2])
            };

            let name_clean = ent.name.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");

            xml.push_str(&format!(
                r#"		<Item class="Part" referent="RBX_{ref_id}">
			<Properties>
				<string name="Name">{name_clean}</string>
				<CoordinateFrame name="CFrame">
					<X>{:.3}</X>
					<Y>{:.3}</Y>
					<Z>{:.3}</Z>
					<R00>1</R00><R01>0</R01><R02>0</R02>
					<R10>0</R10><R11>1</R11><R12>0</R12>
					<R20>0</R20><R21>0</R21><R22>1</R22>
				</CoordinateFrame>
				<Vector3 name="size">
					<X>{:.3}</X>
					<Y>{:.3}</Y>
					<Z>{:.3}</Z>
				</Vector3>
				<Color3 name="Color">
					<R>{:.4}</R>
					<G>{:.4}</G>
					<B>{:.4}</B>
				</Color3>
				<bool name="Anchored">{}</bool>
				<bool name="CanCollide">true</bool>
				<token name="shape">{shape_token}</token>
				<float name="Transparency">0</float>
				<float name="Reflectance">{:.2}</float>
			</Properties>
"#,
                p[0], p[1], p[2],
                sx, sy, sz,
                c[0], c[1], c[2],
                is_anchored,
                ent.material.metallic
            ));

            // If the entity is a light or emissive, add PointLight child item
            if ent.kind == "light" || ent.material.emissive[0] > 0.1 {
                ref_id += 1;
                let light_color = ent.light.as_ref().map(|l| l.color).unwrap_or(ent.material.emissive);
                let brightness = ent.light.as_ref().map(|l| (l.intensity * 0.002).clamp(1.0, 10.0)).unwrap_or(3.0);
                let range = ent.light.as_ref().map(|l| l.range.clamp(8.0, 60.0)).unwrap_or(24.0);

                xml.push_str(&format!(
                    r#"			<Item class="PointLight" referent="RBX_{ref_id}">
				<Properties>
					<string name="Name">PointLight</string>
					<Color3 name="Color">
						<R>{:.3}</R><G>{:.3}</G><B>{:.3}</B>
					</Color3>
					<float name="Brightness">{:.1}</float>
					<float name="Range">{:.1}</float>
					<bool name="Shadows">true</bool>
				</Properties>
			</Item>
"#,
                    light_color[0], light_color[1], light_color[2],
                    brightness, range
                ));
            }

            xml.push_str("		</Item>\n");
        }

        xml.push_str(r#"	</Item>
	<Item class="Lighting" referent="RBX_LIGHTING">
		<Properties>
			<string name="Name">Lighting</string>
			<Color3 name="Ambient"><R>0.45</R><G>0.48</G><B>0.55</B></Color3>
			<float name="Brightness">2.2</float>
			<Color3 name="ColorShift_Top"><R>1.0</R><G>0.95</G><B>0.88</B></Color3>
			<float name="ClockTime">14.5</float>
		</Properties>
	</Item>
	<Item class="ServerScriptService" referent="RBX_SSS">
		<Properties>
			<string name="Name">ServerScriptService</string>
		</Properties>
		<Item class="Script" referent="RBX_MAIN_SCRIPT">
			<Properties>
				<string name="Name">RevEngineGameServer</string>
				<ProtectedString name="Source"><![CDATA[
-- RevEngine Universal Game Server for Roblox Studio
local Players = game:GetService("Players")
Players.PlayerAdded:Connect(function(player)
    local stats = Instance.new("Folder")
    stats.Name = "leaderstats"
    stats.Parent = player
    local score = Instance.new("IntValue")
    score.Name = "Pontos"
    score.Value = 0
    score.Parent = stats
end)
print("◈ RevEngine Game Server ativo no Roblox Studio!")
]]></ProtectedString>
			</Properties>
		</Item>
	</Item>
	<Item class="ReplicatedStorage" referent="RBX_RS">
		<Properties>
			<string name="Name">ReplicatedStorage</string>
		</Properties>
	</Item>
</roblox>
"#);

        xml
    }

    /// Import an existing Roblox .rbxlx XML place into RevEngine Scene
    pub fn import_rbxlx(content: &str, scene: &mut Scene) -> usize {
        let mut count = 0;
        let mut pos = 0;

        while let Some(start_part) = content[pos..].find("<Item class=\"Part\"") {
            let actual_start = pos + start_part;
            let end_part = match content[actual_start..].find("</Item>") {
                Some(e) => actual_start + e + 7,
                None => break,
            };

            let block = &content[actual_start..end_part];

            let name = block
                .split("<string name=\"Name\">")
                .nth(1)
                .and_then(|s| s.split("</string>").next())
                .unwrap_or("RobloxPart")
                .to_string();

            // Extract position CFrame
            let x: f32 = block.split("<X>").nth(1).and_then(|s| s.split("</X>").next()).and_then(|v| v.parse().ok()).unwrap_or(0.0);
            let y: f32 = block.split("<Y>").nth(1).and_then(|s| s.split("</Y>").next()).and_then(|v| v.parse().ok()).unwrap_or(0.0);
            let z: f32 = block.split("<Z>").nth(1).and_then(|s| s.split("</Z>").next()).and_then(|v| v.parse().ok()).unwrap_or(0.0);

            // Extract Size
            let sx: f32 = block.split("<Vector3 name=\"size\">").nth(1).and_then(|s| s.split("<X>").nth(1)).and_then(|s| s.split("</X>").next()).and_then(|v| v.parse().ok()).unwrap_or(2.0);
            let sy: f32 = block.split("<Vector3 name=\"size\">").nth(1).and_then(|s| s.split("<Y>").nth(1)).and_then(|s| s.split("</Y>").next()).and_then(|v| v.parse().ok()).unwrap_or(2.0);
            let sz: f32 = block.split("<Vector3 name=\"size\">").nth(1).and_then(|s| s.split("<Z>").nth(1)).and_then(|s| s.split("</Z>").next()).and_then(|v| v.parse().ok()).unwrap_or(2.0);

            // Extract Color
            let r: f32 = block.split("<Color3 name=\"Color\">").nth(1).and_then(|s| s.split("<R>").nth(1)).and_then(|s| s.split("</R>").next()).and_then(|v| v.parse().ok()).unwrap_or(0.7);
            let g: f32 = block.split("<Color3 name=\"Color\">").nth(1).and_then(|s| s.split("<G>").nth(1)).and_then(|s| s.split("</G>").next()).and_then(|v| v.parse().ok()).unwrap_or(0.7);
            let b: f32 = block.split("<Color3 name=\"Color\">").nth(1).and_then(|s| s.split("<B>").nth(1)).and_then(|s| s.split("</B>").next()).and_then(|v| v.parse().ok()).unwrap_or(0.7);

            let id = scene.alloc_id();
            let mut ent = Entity::new_cube(id, &name, [x, y, z], 1.0, [r, g, b, 1.0]);
            ent.transform.scale = [sx, sy, sz];
            scene.add_entity(ent);
            count += 1;

            pos = end_part;
        }

        count
    }
}
