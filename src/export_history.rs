use crate::model::Batch;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
struct SavedExport {
    folder: PathBuf,
    batch: Batch,
}
fn record_path(data: &Path, folder: &Path) -> PathBuf {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    folder.hash(&mut hash);
    data.join("registros-provas")
        .join(format!("{:016x}.json", hash.finish()))
}
pub fn save(data: &Path, folder: &Path, batch: &Batch) -> Result<(), String> {
    let file = record_path(data, folder);
    fs::create_dir_all(file.parent().unwrap()).map_err(|e| e.to_string())?;
    let record = SavedExport {
        folder: folder.into(),
        batch: batch.clone(),
    };
    fs::write(
        file,
        serde_json::to_vec(&record).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
pub fn read(data: &Path, folder: &Path) -> Result<Batch, String> {
    if let Ok(bytes) = fs::read(record_path(data, folder)) {
        let record: SavedExport = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if record.folder == folder {
            return Ok(record.batch);
        }
    }
    let bytes = fs::read(folder.join("lote.json")).map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}
pub fn list(data: &Path) -> Vec<PathBuf> {
    let mut paths = std::collections::BTreeSet::new();
    for entry in fs::read_dir(data.join("registros-provas"))
        .into_iter()
        .flatten()
        .flatten()
    {
        if let Ok(bytes) = fs::read(entry.path()) {
            if let Ok(record) = serde_json::from_slice::<SavedExport>(&bytes) {
                paths.insert(record.folder);
            }
        }
    }
    for entry in fs::read_dir(data.join("provas"))
        .into_iter()
        .flatten()
        .flatten()
    {
        if entry.path().join("lote.json").exists() {
            paths.insert(entry.path());
        }
    }
    paths.into_iter().rev().collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    #[test]
    fn internal_history_restores_exact_batch_without_extra_export_files() {
        let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("checks/internal-export-history");
        let output = data.join("output");
        fs::create_dir_all(&output).unwrap();
        let mut batch = generate(
            &examples(),
            &[("Mecânica".into(), 1)],
            "Ana\nBruno",
            72,
            "Prova",
            "Escola",
            "Maria",
            "12/11/2026",
            "A",
        )
        .unwrap();
        batch.logo = Some(Picture {
            name: "Logo".into(),
            data: include_bytes!("../assets/test-craft.png").to_vec(),
        });
        save(&data, &output, &batch).unwrap();
        assert_eq!(fs::read_dir(&output).unwrap().count(), 0);
        assert!(list(&data).contains(&output));
        assert_eq!(
            serde_json::to_value(read(&data, &output).unwrap()).unwrap(),
            serde_json::to_value(&batch).unwrap()
        );
        let legacy = data.join("provas/legacy");
        fs::create_dir_all(&legacy).unwrap();
        fs::write(
            legacy.join("lote.json"),
            serde_json::to_vec(&batch).unwrap(),
        )
        .unwrap();
        assert!(read(&data, &legacy).is_ok());
        assert!(list(&data).contains(&legacy));
    }
}
