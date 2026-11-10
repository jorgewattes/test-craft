use crate::model::{categories, category_matches, validate, Bank, Question};
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc, Arc,
    },
};

pub const PAGE_SIZE: usize = 50;

/// Lightweight lookup/search data; never clone images or rebuild text on repaint.
pub struct BankIndex {
    pub positions: BTreeMap<u64, usize>,
    pub categories: Vec<String>,
    pub category_counts: BTreeMap<String, usize>,
    search: Vec<String>,
    filter: Option<(String, String)>,
    matches: Arc<Vec<usize>>,
}
impl BankIndex {
    pub fn new(bank: &Bank) -> Self {
        let mut category_counts = BTreeMap::new();
        for q in &bank.questions {
            let mut path = q.category.as_str();
            loop {
                *category_counts.entry(path.to_owned()).or_default() += 1;
                match path.rsplit_once('/') {
                    Some((parent, _)) => path = parent,
                    None => break,
                }
            }
        }
        Self {
            category_counts,
            positions: bank
                .questions
                .iter()
                .enumerate()
                .map(|(i, q)| (q.id, i))
                .collect(),
            categories: categories(bank),
            search: bank
                .questions
                .iter()
                .map(|q| format!("{} {} {}", q.title, q.category, q.text).to_lowercase())
                .collect(),
            filter: None,
            matches: Arc::new(Vec::new()),
        }
    }
    pub fn filter(&mut self, bank: &Bank, category: &str, search: &str) -> (Arc<Vec<usize>>, bool) {
        let key = (category.to_owned(), search.to_lowercase());
        let changed = self.filter.as_ref() != Some(&key);
        if changed {
            self.matches = Arc::new(
                bank.questions
                    .iter()
                    .enumerate()
                    .filter(|(i, q)| {
                        (category == "Todas" || category_matches(&q.category, category))
                            && self.search[*i].contains(&key.1)
                    })
                    .map(|(i, _)| i)
                    .collect(),
            );
            self.filter = Some(key);
        }
        (self.matches.clone(), changed)
    }
}

pub struct ValidationResult {
    pub generation: u64,
    pub id: u64,
    pub error: Option<String>,
}
pub struct Validator {
    requests: mpsc::Sender<(u64, Vec<Question>)>,
    pub results: mpsc::Receiver<ValidationResult>,
    generation: Arc<AtomicU64>,
}
impl Validator {
    pub fn new(ctx: eframe::egui::Context) -> Self {
        let (requests, input) = mpsc::channel::<(u64, Vec<Question>)>();
        let (output, results) = mpsc::channel();
        let generation = Arc::new(AtomicU64::new(0));
        let current = generation.clone();
        std::thread::spawn(move || {
            let mut cache: BTreeMap<u64, (Question, Option<String>)> = BTreeMap::new();
            while let Ok(mut request) = input.recv() {
                // Discard superseded batches; keep a single serial validator.
                while let Ok(newer) = input.try_recv() {
                    request = newer;
                }
                let (generation, questions) = request;
                let ids: std::collections::BTreeSet<_> = questions.iter().map(|q| q.id).collect();
                cache.retain(|id, _| ids.contains(id));
                for q in questions {
                    if current.load(Ordering::Relaxed) != generation {
                        break;
                    }
                    let error = match cache.get(&q.id) {
                        Some((old, error)) if old == &q => error.clone(),
                        _ => validate(&q).err(),
                    };
                    let id = q.id;
                    cache.insert(id, (q, error.clone()));
                    if output
                        .send(ValidationResult {
                            generation,
                            id,
                            error,
                        })
                        .is_err()
                    {
                        return;
                    }
                    ctx.request_repaint();
                }
            }
        });
        Self {
            requests,
            results,
            generation,
        }
    }
    pub fn submit(&self, bank: &Bank) -> u64 {
        let generation = self.generation.fetch_add(1, Ordering::Relaxed) + 1;
        let _ = self.requests.send((generation, bank.questions.clone()));
        generation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cached_search_matches_legacy_filters_and_rebuilds_after_changes() {
        let mut bank = crate::model::examples();
        bank.questions[0].category = "Física/Óptica".into();
        let mut index = BankIndex::new(&bank);
        for category in ["Todas", "Física", "Física/Óptica", "Ausente"] {
            for search in ["", "FÍSICA", "velocidade", "Ausente"] {
                let (actual, _) = index.filter(&bank, category, search);
                let expected: Vec<_> = bank
                    .questions
                    .iter()
                    .enumerate()
                    .filter(|(_, q)| {
                        (category == "Todas" || category_matches(&q.category, category))
                            && format!("{} {} {}", q.title, q.category, q.text)
                                .to_lowercase()
                                .contains(&search.to_lowercase())
                    })
                    .map(|(i, _)| i)
                    .collect();
                assert_eq!(*actual, expected);
                let (same, changed) = index.filter(&bank, category, search);
                assert!(!changed);
                assert!(Arc::ptr_eq(&actual, &same));
            }
        }
        bank.questions.remove(0);
        let mut index = BankIndex::new(&bank);
        assert!(index.filter(&bank, "Física", "").0.is_empty());
    }
    #[test]
    fn validator_returns_latest_revision_and_removes_fixed_errors() {
        let worker = Validator::new(eframe::egui::Context::default());
        let mut bank = crate::model::examples();
        bank.questions.truncate(1);
        bank.questions[0].title.clear();
        let first = worker.submit(&bank);
        let result = worker
            .results
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        assert_eq!(result.generation, first);
        assert!(result.error.is_some());
        // Exercise a superseded request, then a correction using the same id.
        worker.submit(&bank);
        bank.questions[0].title = "Corrigida".into();
        let latest = worker.submit(&bank);
        loop {
            let result = worker
                .results
                .recv_timeout(std::time::Duration::from_secs(10))
                .unwrap();
            if result.generation == latest {
                assert!(result.error.is_none());
                break;
            }
        }
    }
}
