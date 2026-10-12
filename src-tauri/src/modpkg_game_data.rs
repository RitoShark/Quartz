use std::fs;
use std::path::Path;

/// Restores metadata-only declarations that the older project importer cannot read.
pub(super) fn restore(archive: &Path, project: &Path) -> Result<(), String> {
    let file = fs::File::open(archive).map_err(|error| error.to_string())?;
    let mut package =
        ltk_modpkg::Modpkg::mount_from_reader(file).map_err(|error| error.to_string())?;
    let bytes = package
        .load_chunk_decompressed_by_path("_meta_/info.msgpack", None)
        .map_err(|error| error.to_string())?;
    let metadata: serde_json::Value =
        rmp_serde::from_slice(&bytes).map_err(|error| error.to_string())?;
    for layer in metadata["layers"].as_array().into_iter().flatten() {
        let Some(name) = layer["name"].as_str() else {
            continue;
        };
        if name.is_empty()
            || !name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        {
            return Err(format!("Invalid declaration layer: {name}"));
        }
        let directory = project.join("content").join(name);
        // Authored files take precedence and keep their comments and source layout.
        if ltk_game_data::MANIFEST_NAMES
            .iter()
            .any(|name| directory.join(name).is_file())
        {
            continue;
        }
        let Some(value) = layer.get("game_data").filter(|value| !value.is_null()) else {
            continue;
        };
        let document: ltk_game_data::DeclarationDocument =
            serde_json::from_value(value.clone()).map_err(|error| format!("{name}: {error}"))?;
        let manifest = document
            .parse()
            .and_then(|declarations| declarations.manifest_json())
            .map_err(|error| format!("{name}: {error}"))?;
        fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        fs::write(directory.join("game_data.json"), manifest).map_err(|error| error.to_string())?;
    }
    Ok(())
}
