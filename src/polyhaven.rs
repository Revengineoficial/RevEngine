use serde_json::Value;
use std::{collections::BTreeMap, fs, path::Path};

const API: &str = "https://api.polyhaven.com";

#[derive(Clone, Debug)]
pub struct FreeAsset {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub license: String,
}

pub fn search(query: &str, kind: &str) -> Result<Vec<FreeAsset>, String> {
    let asset_type = match kind {
        "Texturas" => "textures",
        "HDRIs" => "hdris",
        _ => "models",
    };
    let response: Value = reqwest::blocking::get(format!("{API}/assets?t={asset_type}"))
        .map_err(|e| format!("Falha ao consultar Poly Haven: {e}"))?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| format!("Resposta inválida do catálogo: {e}"))?;
    let needle = query.trim().to_lowercase();
    let mut results = Vec::new();
    if let Some(items) = response.as_object() {
        for (id, item) in items {
            let name = item
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or(id)
                .to_string();
            if needle.is_empty()
                || id.to_lowercase().contains(&needle)
                || name.to_lowercase().contains(&needle)
            {
                results.push(FreeAsset {
                    id: id.clone(),
                    name,
                    kind: kind.into(),
                    license: "CC0 — domínio público (conforme catálogo Poly Haven)".into(),
                });
            }
        }
    }
    results.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    results.truncate(100);
    Ok(results)
}

fn collect_urls(value: &Value, path: &str, found: &mut BTreeMap<String, String>) {
    match value {
        Value::Object(map) => {
            if let Some(url) = map.get("url").and_then(Value::as_str) {
                if url.starts_with("https://") {
                    found.insert(path.to_lowercase(), url.to_string());
                }
            }
            for (key, child) in map {
                let next = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}/{key}")
                };
                collect_urls(child, &next, found);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                collect_urls(child, &format!("{path}/{index}"), found);
            }
        }
        _ => {}
    }
}

fn safe_asset_dir(project: &Path, kind: &str, id: &str) -> Result<std::path::PathBuf, String> {
    if id.is_empty()
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err("ID de asset inválido retornado pelo catálogo.".into());
    }
    let folder = match kind {
        "Texturas" => "Textures",
        "HDRIs" => "HDRI",
        _ => "Models",
    };
    let base = project.join("Assets").join(folder).join(id);
    fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    Ok(base)
}

fn download_url(url: &str, path: &Path) -> Result<(), String> {
    let response = reqwest::blocking::get(url)
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    let bytes = response.bytes().map_err(|e| e.to_string())?;
    if bytes.is_empty() {
        return Err("O servidor retornou um arquivo vazio.".into());
    }
    fs::write(path, &bytes).map_err(|e| e.to_string())
}

pub fn download(asset: &FreeAsset, project: &Path) -> Result<Vec<String>, String> {
    let files: Value = reqwest::blocking::get(format!("{API}/files/{}", asset.id))
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;
    let base = safe_asset_dir(project, &asset.kind, &asset.id)?;
    let mut urls = BTreeMap::new();
    collect_urls(&files, "", &mut urls);
    let mut saved = Vec::new();

    if asset.kind == "Modelos" {
        let (path_key, url) = urls
            .iter()
            .find(|(key, _)| key.contains("gltf"))
            .or_else(|| urls.iter().find(|(key, _)| key.contains("glb")))
            .ok_or("A API não retornou uma variante glTF/GLB para este modelo.")?;
        let ext = if path_key.contains("glb") {
            "glb"
        } else {
            "gltf"
        };
        let model_path = base.join(format!("{}.{}", asset.id, ext));
        download_url(url, &model_path)?;
        if ext == "gltf" {
            let document: Value =
                serde_json::from_slice(&fs::read(&model_path).map_err(|e| e.to_string())?)
                    .map_err(|e| format!("glTF inválido: {e}"))?;
            let base_url = reqwest::Url::parse(url).map_err(|e| e.to_string())?;
            for uri in document
                .get("buffers")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .chain(
                    document
                        .get("images")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten(),
                )
                .filter_map(|entry| entry.get("uri").and_then(Value::as_str))
            {
                if uri.starts_with("data:") {
                    continue;
                }
                let relative = Path::new(uri);
                if relative.is_absolute()
                    || relative
                        .components()
                        .any(|c| !matches!(c, std::path::Component::Normal(_)))
                {
                    return Err(format!("URI de dependência glTF insegura: {uri}"));
                }
                let dep_url = base_url.join(uri).map_err(|e| e.to_string())?;
                let dep_path = base.join(relative);
                if let Some(parent) = dep_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                download_url(dep_url.as_str(), &dep_path)?;
                saved.push(dep_path.to_string_lossy().into_owned());
            }
        }
        saved.push(model_path.to_string_lossy().into_owned());
    } else if asset.kind == "Texturas" {
        // Download all available 1K PBR channels, not just one preview image.
        let channels = [
            "diff", "albedo", "color", "rough", "metal", "nor_gl", "nor_dx", "ao", "disp", " bump",
        ];
        for channel in channels {
            let candidate = urls.iter().find(|(key, _)| {
                let k = key.as_str();
                (k.contains("1k") || k.contains("2k")) && k.contains(channel.trim())
            });
            if let Some((key, url)) = candidate {
                let ext = url
                    .split('?')
                    .next()
                    .and_then(|u| u.rsplit('.').next())
                    .filter(|e| e.len() <= 5)
                    .unwrap_or("bin");
                let label = key
                    .split('/')
                    .find(|part| part.contains(channel.trim()))
                    .unwrap_or(channel.trim());
                let path = base.join(format!(
                    "{}_{}.{}",
                    asset.id,
                    label.replace(['/', ' '], "_"),
                    ext
                ));
                if download_url(url, &path).is_ok() {
                    saved.push(path.to_string_lossy().into_owned());
                }
            }
        }
        if saved.is_empty() {
            let (key, url) = urls
                .iter()
                .find(|(key, _)| key.contains("1k"))
                .or_else(|| urls.iter().next())
                .ok_or("Nenhum mapa de textura compatível foi encontrado.")?;
            let ext = url
                .split('?')
                .next()
                .and_then(|u| u.rsplit('.').next())
                .filter(|e| e.len() <= 5)
                .unwrap_or("bin");
            let path = base.join(format!("{}.{}", asset.id, ext));
            download_url(url, &path)?;
            let _ = key;
            saved.push(path.to_string_lossy().into_owned());
        }
    } else {
        let (key, url) = urls
            .iter()
            .find(|(key, _)| key.contains("hdr") && (key.contains("1k") || key.contains("2k")))
            .or_else(|| urls.iter().next())
            .ok_or("Nenhum arquivo HDRI compatível foi encontrado.")?;
        let ext = url
            .split('?')
            .next()
            .and_then(|u| u.rsplit('.').next())
            .filter(|e| e.len() <= 5)
            .unwrap_or("exr");
        let path = base.join(format!("{}.{}", asset.id, ext));
        download_url(url, &path)?;
        let _ = key;
        saved.push(path.to_string_lossy().into_owned());
    }

    let license = format!(
        "Asset: {}\nID: {}\nFonte: Poly Haven\nLicença declarada pelo catálogo: CC0\nPágina do asset: https://polyhaven.com/a/{}\nVerifique a página e os termos atuais antes de redistribuir.\n",
        asset.name, asset.id, asset.id
    );
    fs::write(base.join("LICENSE.txt"), license).map_err(|e| e.to_string())?;
    Ok(saved)
}
