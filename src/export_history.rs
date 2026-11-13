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
    #[derive(Serialize)]
    struct ExportRef<'a> {
        folder: &'a Path,
        batch: &'a Batch,
    }
    let record = ExportRef { folder, batch };
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
/// Cached list data. Loading the full batch (including images) is reserved for preview.
#[derive(Clone, Debug)]
pub struct HistoryEntry {
    pub folder: PathBuf,
    pub title: String,
    pub student_count: usize,
    pub seed: u64,
}
#[derive(Deserialize)]
struct BatchSummary {
    title: String,
    seed: u64,
    students: Vec<serde::de::IgnoredAny>,
}
#[derive(Deserialize)]
struct ExportSummary {
    folder: PathBuf,
    batch: BatchSummary,
}
impl HistoryEntry {
    fn new(folder: PathBuf, batch: BatchSummary) -> Self {
        Self { folder, title: batch.title, student_count: batch.students.len(), seed: batch.seed }
    }
}
pub fn list(data: &Path) -> Vec<HistoryEntry> {
    let mut entries = std::collections::BTreeMap::new();
    for entry in fs::read_dir(data.join("registros-provas"))
        .into_iter().flatten().flatten()
    {
        if let Ok(bytes) = fs::read(entry.path()) {
            if let Ok(record) = serde_json::from_slice::<ExportSummary>(&bytes) {
                entries.insert(record.folder.clone(), HistoryEntry::new(record.folder, record.batch));
            }
        }
    }
    for entry in fs::read_dir(data.join("provas"))
        .into_iter().flatten().flatten()
    {
        let folder = entry.path();
        // The internal record takes precedence over a legacy export for the same folder.
        if !entries.contains_key(&folder) {
            if let Ok(bytes) = fs::read(folder.join("lote.json")) {
                if let Ok(batch) = serde_json::from_slice::<BatchSummary>(&bytes) {
                    entries.insert(folder.clone(), HistoryEntry::new(folder, batch));
                }
            }
        }
    }
    entries.into_values().rev().collect()
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
        let entries = list(&data);
        let entry = entries.iter().find(|entry| entry.folder == output).unwrap();
        assert_eq!(entry.title, batch.title);
        assert_eq!(entry.student_count, batch.students.len());
        assert_eq!(entry.seed, batch.seed);
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
        assert!(list(&data).iter().any(|entry| entry.folder == legacy));
        // Refresh creates a snapshot: painting it needs no files or full batch payload.
        let cached = list(&data);
        fs::write(record_path(&data, &output), b"invalid").unwrap();
        assert!(cached.iter().any(|entry| entry.folder == output && entry.student_count == 2));
        assert!(!list(&data).iter().any(|entry| entry.folder == output));
        // Full preview remains strict about invalid records.
        assert!(read(&data, &output).is_err());
        save(&data, &output, &batch).unwrap();
        save(&data, &legacy, &batch).unwrap();
        assert_eq!(list(&data).iter().filter(|entry| entry.folder == legacy).count(), 1);
    }
}
