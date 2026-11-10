//! Versioned local database. JSON remains the interchange/legacy format.
use crate::model::{categories, examples, read_bank, Bank, Question};
use rusqlite::{params, Connection, TransactionBehavior};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    time::Duration,
};

pub struct BankStore {
    connection: Connection,
    revision: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn memory() -> BankStore {
        BankStore::configure(Connection::open_in_memory().unwrap()).unwrap()
    }
    fn folder(label: &str) -> std::path::PathBuf {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("checks")
            .join(format!(
                "storage-{label}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }
    #[test]
    fn migration_keeps_legacy_and_reopens_full_payload_and_empty_categories() {
        let dir = folder("migration");
        let mut bank = examples();
        bank.category_labels.push("Vazia/Subcategoria".into());
        bank.questions[0].id = u64::MAX;
        bank.questions[0].source_xml = Some("<question>original &amp; conteúdo</question>".into());
        bank.questions[0].pictures.push(crate::model::Picture {
            name: "foto".into(),
            data: vec![0, 255, 8],
        });
        bank.questions[0].import_issues.push("Revisar".into());
        bank.questions[1].category.clear(); // Invalid drafts remain saveable.
        bank.questions[2].category = "/Legado/Subcategoria".into();
        let legacy = dir.join("banco.json");
        crate::model::save_bank(&legacy, &bank).unwrap();
        let bytes = std::fs::read(&legacy).unwrap();
        let (store, reopened) = BankStore::open(&dir).unwrap();
        assert_eq!(reopened.questions, bank.questions);
        assert!(reopened
            .category_labels
            .contains(&"Vazia/Subcategoria".into()));
        assert_eq!(std::fs::read(&legacy).unwrap(), bytes);
        drop(store);
        // Once migrated, the database is authoritative; the preserved JSON is a backup.
        std::fs::write(&legacy, b"old backup").unwrap();
        let (_, again) = BankStore::open(&dir).unwrap();
        assert_eq!(again.questions, bank.questions);
    }
    #[test]
    fn invalid_legacy_is_preserved_and_never_replaced_with_examples() {
        let dir = folder("invalid");
        std::fs::write(dir.join("banco.json"), b"invalid").unwrap();
        assert!(BankStore::open(&dir).is_err());
        assert!(BankStore::open(&dir).is_err());
        assert_eq!(std::fs::read(dir.join("banco.json")).unwrap(), b"invalid");
    }
    #[test]
    fn one_edit_in_ten_thousand_questions_writes_one_payload() {
        let mut store = memory();
        let mut bank = examples();
        let template = bank.questions[0].clone();
        bank.questions = (0..10000)
            .map(|id| {
                let mut q = template.clone();
                q.id = id;
                q
            })
            .collect();
        store.save(&bank, None).unwrap();
        let previous = bank.clone();
        bank.questions[5000].title = "Atualizada".into();
        let before = store.connection.total_changes();
        let start = std::time::Instant::now();
        store.save(&bank, Some(&previous)).unwrap();
        // One payload plus one metadata revision; no whole-table rewrites.
        assert_eq!(store.connection.total_changes() - before, 2);
        println!(
            "10,000 questions: one edit committed in {:?}",
            start.elapsed()
        );
        assert_eq!(store.load().unwrap().questions, bank.questions);
    }
    #[test]
    fn category_move_copy_delete_order_and_transaction_rollback() {
        let mut store = memory();
        let mut bank = examples();
        store.save(&bank, None).unwrap();
        let old = bank.clone();
        bank.category_labels.push("Destino/Sub".into());
        let ids = BTreeSet::from([bank.questions[0].id]);
        crate::model::transfer_questions(&mut bank, &ids, "Destino/Sub", true).unwrap();
        crate::model::transfer_questions(&mut bank, &ids, "Destino/Sub", false).unwrap();
        bank.questions.remove(1);
        bank.questions.reverse();
        store.save(&bank, Some(&old)).unwrap();
        assert_eq!(store.load().unwrap().questions, bank.questions);
        let old = bank.clone();
        bank.questions.remove(0);
        let mut q = old.questions[0].clone();
        q.id = 99;
        bank.questions.push(q);
        store.connection.execute_batch("CREATE TRIGGER fail_insert BEFORE INSERT ON questions WHEN NEW.id='99' BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
        assert!(store.save(&bank, Some(&old)).is_err());
        assert_eq!(store.load().unwrap().questions, old.questions);
        let integrity: String = store
            .connection
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap();
        assert_eq!(integrity, "ok");
        assert_eq!(
            store
                .connection
                .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| r
                    .get::<_, i64>(
                    0
                ))
                .unwrap(),
            0
        );
    }
    #[test]
    fn stale_window_cannot_overwrite_newer_commit() {
        let dir = folder("concurrent");
        let (mut first, old) = BankStore::open(&dir).unwrap();
        let (mut second, mut stale) = BankStore::open(&dir).unwrap();
        let mut newer = old.clone();
        newer.questions[0].title = "Primeira janela".into();
        first.save(&newer, Some(&old)).unwrap();
        stale.questions[0].title = "Janela desatualizada".into();
        assert!(second
            .save(&stale, Some(&old))
            .unwrap_err()
            .contains("outra janela"));
        assert_eq!(second.load().unwrap().questions, newer.questions);
    }
}

impl BankStore {
    pub fn open(directory: &Path) -> Result<(Self, Bank), String> {
        std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
        let connection =
            Connection::open(directory.join("banco.sqlite3")).map_err(|e| e.to_string())?;
        let mut store = Self::configure(connection)?;
        let initialized: bool = store
            .connection
            .query_row(
                "SELECT initialized FROM bank_meta WHERE singleton = 1",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if !initialized {
            let legacy = directory.join("banco.json");
            // Never replace an unreadable bank with examples. Keep the legacy file intact.
            let bank = if legacy.exists() {
                read_bank(&legacy)?
            } else {
                examples()
            };
            store.save(&bank, None)?;
        }
        let bank = store.load()?;
        store
            .connection
            .execute_batch("PRAGMA optimize=0x10002;")
            .map_err(|e| e.to_string())?;
        Ok((store, bank))
    }

    fn configure(connection: Connection) -> Result<Self, String> {
        // A second application must fail promptly instead of freezing the UI on a lock.
        connection
            .busy_timeout(Duration::from_millis(150))
            .map_err(|e| e.to_string())?;
        connection
            .execute_batch(
                "PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA wal_autocheckpoint=256; PRAGMA journal_size_limit=8388608;",
            )
            .map_err(|e| e.to_string())?;
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if version > 1 {
            return Err(
                "Versão do banco mais recente que este aplicativo. Atualize o Test Craft.".into(),
            );
        }
        connection
            .execute_batch(
                "
            BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS bank_meta (
                singleton INTEGER PRIMARY KEY CHECK(singleton=1),
                revision INTEGER NOT NULL, initialized INTEGER NOT NULL CHECK(initialized IN (0,1))
            );
            INSERT OR IGNORE INTO bank_meta VALUES (1,0,0);
            CREATE TABLE IF NOT EXISTS categories (
                path TEXT PRIMARY KEY,
                parent TEXT REFERENCES categories(path) DEFERRABLE INITIALLY DEFERRED
            );
            CREATE TABLE IF NOT EXISTS questions (
                id TEXT PRIMARY KEY, position INTEGER NOT NULL CHECK(position>=0),
                category TEXT NOT NULL REFERENCES categories(path) DEFERRABLE INITIALLY DEFERRED,
                payload BLOB NOT NULL
            );
            CREATE INDEX IF NOT EXISTS questions_category_position ON questions(category, position);
            CREATE INDEX IF NOT EXISTS questions_position ON questions(position);
            PRAGMA user_version=1;
            COMMIT;
        ",
            )
            .map_err(|e| e.to_string())?;
        Ok(Self {
            connection,
            revision: 0,
        })
    }

    fn load(&mut self) -> Result<Bank, String> {
        let tx = self.connection.transaction().map_err(|e| e.to_string())?;
        let revision = tx
            .query_row(
                "SELECT revision FROM bank_meta WHERE singleton=1",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        let mut questions = Vec::new();
        {
            let mut stmt = tx
                .prepare("SELECT id, category, payload FROM questions ORDER BY position")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, Vec<u8>>(2)?,
                    ))
                })
                .map_err(|e| e.to_string())?;
            for row in rows {
                let (id, category, payload) = row.map_err(|e| e.to_string())?;
                let q: Question = serde_json::from_slice(&payload).map_err(|e| e.to_string())?;
                if q.id.to_string() != id || q.category != category {
                    return Err(
                        "Banco inconsistente: identificação ou categoria divergente.".into(),
                    );
                }
                questions.push(q);
            }
        }
        let labels = {
            let mut stmt = tx
                .prepare("SELECT path FROM categories WHERE path<>'' ORDER BY path")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |r| r.get::<_, String>(0))
                .map_err(|e| e.to_string())?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?
        };
        tx.commit().map_err(|e| e.to_string())?;
        self.revision = revision;
        Ok(Bank {
            version: 1,
            questions,
            category_labels: labels,
        })
    }

    /// Commit the complete operation atomically, writing only changed question payloads.
    pub fn save(&mut self, bank: &Bank, previous: Option<&Bank>) -> Result<(), String> {
        if bank.version != 1 || bank.questions.len() > 10000 {
            return Err("Banco incompatível ou com mais de 10.000 questões.".into());
        }
        let ids: BTreeSet<_> = bank.questions.iter().map(|q| q.id).collect();
        if ids.len() != bank.questions.len() {
            return Err("Banco com identificadores repetidos.".into());
        }
        let old: BTreeMap<_, _> = previous
            .into_iter()
            .flat_map(|b| b.questions.iter().enumerate())
            .map(|(i, q)| (q.id, (i, q)))
            .collect();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|e| e.to_string())?;
        let revision: i64 = tx
            .query_row(
                "SELECT revision FROM bank_meta WHERE singleton=1",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if revision != self.revision {
            return Err(
                "O banco foi alterado em outra janela. Reabra o aplicativo antes de salvar.".into(),
            );
        }
        let old_labels: Option<BTreeSet<String>> = previous.map(|bank| {
            categories(bank)
                .into_iter()
                .chain(bank.questions.iter().map(|q| q.category.clone()))
                .collect()
        });
        let mut labels: BTreeSet<_> = categories(bank).into_iter().collect();
        labels.extend(bank.questions.iter().map(|q| q.category.clone()));
        {
            let mut insert = tx
                .prepare_cached("INSERT OR IGNORE INTO categories(path,parent) VALUES (?1,?2)")
                .map_err(|e| e.to_string())?;
            for path in labels
                .iter()
                .filter(|p| old_labels.as_ref().is_none_or(|old| !old.contains(*p)))
            {
                let parent = path
                    .rsplit_once('/')
                    .map(|(p, _)| p)
                    .filter(|p| !p.is_empty());
                insert
                    .execute(params![path, parent])
                    .map_err(|e| e.to_string())?;
            }
        }
        for id in old.keys().filter(|id| !ids.contains(id)) {
            tx.execute("DELETE FROM questions WHERE id=?1", [id.to_string()])
                .map_err(|e| e.to_string())?;
        }
        {
            let mut upsert = tx.prepare_cached("INSERT INTO questions(id,position,category,payload) VALUES (?1,?2,?3,?4)
                ON CONFLICT(id) DO UPDATE SET position=excluded.position,category=excluded.category,payload=excluded.payload").map_err(|e| e.to_string())?;
            let mut position = tx
                .prepare_cached("UPDATE questions SET position=?2 WHERE id=?1")
                .map_err(|e| e.to_string())?;
            for (i, q) in bank.questions.iter().enumerate() {
                match old.get(&q.id) {
                    Some((old_i, old_q)) if *old_q == q => {
                        if *old_i != i {
                            position
                                .execute(params![q.id.to_string(), i as i64])
                                .map_err(|e| e.to_string())?;
                        }
                    }
                    _ => {
                        let payload = serde_json::to_vec(q).map_err(|e| e.to_string())?;
                        upsert
                            .execute(params![q.id.to_string(), i as i64, q.category, payload])
                            .map_err(|e| e.to_string())?;
                    }
                }
            }
        }
        let existing = if let Some(old) = old_labels {
            old.into_iter().collect::<Vec<_>>()
        } else {
            let mut stmt = tx
                .prepare("SELECT path FROM categories")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |r| r.get::<_, String>(0))
                .map_err(|e| e.to_string())?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?
        };
        for path in existing.iter().filter(|p| !labels.contains(*p)) {
            tx.execute("DELETE FROM categories WHERE path=?1", [path])
                .map_err(|e| e.to_string())?;
        }
        tx.execute(
            "UPDATE bank_meta SET revision=revision+1, initialized=1 WHERE singleton=1",
            [],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        self.revision += 1;
        Ok(())
    }
}

impl Drop for BankStore {
    fn drop(&mut self) {
        let _ = self
            .connection
            .execute_batch("PRAGMA optimize; PRAGMA wal_checkpoint(PASSIVE);");
    }
}
