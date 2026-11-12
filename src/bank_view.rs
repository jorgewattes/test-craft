use crate::model::{
    categories, category_matches, validate_with_example, Bank, Question, QuestionStatus,
    ValidationExample,
};
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc, Arc,
    },
};

pub const PAGE_SIZE: usize = 50;

pub fn category_counts(bank: &Bank) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for q in &bank.questions {
        let mut path = q.category.as_str();
        loop {
            *counts.entry(path.to_owned()).or_default() += 1;
            match path.rsplit_once('/') {
                Some((parent, _)) => path = parent,
                None => break,
            }
        }
    }
    counts
}

pub fn category_search_matches(path: &str, query: &str) -> bool {
    fn normalized(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .map(|c| match c {
                'á' | 'à' | 'â' | 'ã' => 'a',
                'é' | 'ê' => 'e',
                'í' => 'i',
                'ó' | 'ô' | 'õ' => 'o',
                'ú' | 'ü' => 'u',
                'ç' => 'c',
                _ => c,
            })
            .collect()
    }
    let path = normalized(path);
    normalized(query)
        .split_whitespace()
        .all(|term| path.contains(term))
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum QuestionFilter {
    #[default]
    All,
    Ready,
    Error,
    Draft,
}
impl QuestionFilter {
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "Todas",
            Self::Ready => "Prontas",
            Self::Error => "Com erro",
            Self::Draft => "Rascunhos",
        }
    }
}

/// Lightweight lookup/search data; never clone images or rebuild text on repaint.
pub struct BankIndex {
    pub positions: BTreeMap<u64, usize>,
    pub categories: Vec<String>,
    pub category_counts: BTreeMap<String, usize>,
    pub ready_counts: BTreeMap<String, usize>,
    search: Vec<String>,
    filter: Option<(String, String, QuestionFilter)>,
    matches: Arc<Vec<usize>>,
}
impl BankIndex {
    pub fn new(bank: &Bank) -> Self {
        Self {
            category_counts: category_counts(bank),
            ready_counts: BTreeMap::new(),
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
    pub fn invalidate_filter(&mut self) {
        self.filter = None;
    }
    pub fn available_by_category(&self, plan: &[(String, usize)]) -> BTreeMap<String, usize> {
        let added: std::collections::BTreeSet<_> = plan.iter().map(|(path, _)| path).collect();
        let mut counts = BTreeMap::new();
        for (path, count) in &self.ready_counts {
            if added.contains(path) {
                continue;
            }
            let mut path = path.as_str();
            loop {
                *counts.entry(path.to_owned()).or_default() += count;
                match path.rsplit_once('/') {
                    Some((parent, _)) => path = parent,
                    None => break,
                }
            }
        }
        counts
    }
    /// Expand a parent into disjoint direct-category pools. Existing plan rows
    /// keep their quantities and no question can be sampled twice via a parent.
    pub fn add_category_tree(&self, root: &str, plan: &mut Vec<(String, usize)>) {
        let mut added: std::collections::BTreeSet<_> =
            plan.iter().map(|(path, _)| path.clone()).collect();
        for path in &self.categories {
            if category_matches(path, root)
                && self.ready_counts.get(path).copied().unwrap_or(0) > 0
                && added.insert(path.clone())
            {
                plan.push((path.clone(), 1));
            }
        }
    }
    pub fn filter(
        &mut self,
        bank: &Bank,
        category: &str,
        search: &str,
        status: QuestionFilter,
        errors: &BTreeMap<u64, String>,
        pending: &std::collections::BTreeSet<u64>,
    ) -> (Arc<Vec<usize>>, bool) {
        let key = (category.to_owned(), search.trim().to_lowercase(), status);
        let changed = self.filter.as_ref() != Some(&key);
        if changed {
            self.matches = Arc::new(
                bank.questions
                    .iter()
                    .enumerate()
                    .filter(|(i, q)| {
                        (category == "Todas" || category_matches(&q.category, category))
                            && self.search[*i].contains(&key.1)
                            && match status {
                                QuestionFilter::All => true,
                                QuestionFilter::Draft => q.status == QuestionStatus::Draft,
                                QuestionFilter::Ready => {
                                    q.status == QuestionStatus::Ready
                                        && !pending.contains(&q.id)
                                        && !errors.contains_key(&q.id)
                                }
                                QuestionFilter::Error => {
                                    !pending.contains(&q.id) && errors.contains_key(&q.id)
                                }
                            }
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
    pub example: Option<ValidationExample>,
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
            let mut cache: BTreeMap<u64, (Question, Option<String>, Option<ValidationExample>)> =
                BTreeMap::new();
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
                    let (error, example) = match cache.get(&q.id) {
                        Some((old, error, example)) if old == &q => {
                            (error.clone(), example.clone())
                        }
                        _ => {
                            let (result, example) = validate_with_example(&q);
                            (result.err(), example)
                        }
                    };
                    let id = q.id;
                    cache.insert(id, (q, error.clone(), example.clone()));
                    if output
                        .send(ValidationResult {
                            generation,
                            id,
                            error,
                            example,
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
                let (actual, _) = index.filter(
                    &bank,
                    category,
                    search,
                    QuestionFilter::All,
                    &BTreeMap::new(),
                    &Default::default(),
                );
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
                let (same, changed) = index.filter(
                    &bank,
                    category,
                    search,
                    QuestionFilter::All,
                    &BTreeMap::new(),
                    &Default::default(),
                );
                assert!(!changed);
                assert!(Arc::ptr_eq(&actual, &same));
            }
        }
        bank.questions.remove(0);
        let mut index = BankIndex::new(&bank);
        assert!(index
            .filter(
                &bank,
                "Física",
                "",
                QuestionFilter::All,
                &BTreeMap::new(),
                &Default::default()
            )
            .0
            .is_empty());
    }
    #[test]
    fn status_filters_combine_with_search_and_follow_validation_updates() {
        let mut bank = crate::model::examples();
        bank.questions.truncate(3);
        bank.questions[1].status = QuestionStatus::Draft;
        bank.questions[2].text.clear();
        let mut errors = BTreeMap::from([(bank.questions[2].id, "Enunciado vazio".into())]);
        let mut pending = std::collections::BTreeSet::from([bank.questions[0].id]);
        let mut index = BankIndex::new(&bank);
        assert!(index
            .filter(&bank, "Todas", "", QuestionFilter::Ready, &errors, &pending)
            .0
            .is_empty());
        assert_eq!(
            *index
                .filter(&bank, "Todas", "", QuestionFilter::Draft, &errors, &pending)
                .0,
            vec![1]
        );
        assert_eq!(
            *index
                .filter(&bank, "Todas", "", QuestionFilter::Error, &errors, &pending)
                .0,
            vec![2]
        );
        assert!(index
            .filter(
                &bank,
                "Ausente",
                "",
                QuestionFilter::Error,
                &errors,
                &pending
            )
            .0
            .is_empty());
        assert!(index
            .filter(
                &bank,
                "Todas",
                "não existe",
                QuestionFilter::Draft,
                &errors,
                &pending
            )
            .0
            .is_empty());
        pending.clear();
        index.invalidate_filter();
        assert_eq!(
            *index
                .filter(&bank, "Todas", "", QuestionFilter::Ready, &errors, &pending)
                .0,
            vec![0]
        );
        errors.clear();
        index.invalidate_filter();
        assert!(index
            .filter(&bank, "Todas", "", QuestionFilter::Error, &errors, &pending)
            .0
            .is_empty());
    }
    #[test]
    fn category_search_accepts_accents_spaces_and_parent_paths() {
        assert!(category_search_matches(
            "Ciências/Física/Óptica",
            "  FISICA optica  "
        ));
        assert!(category_search_matches(
            "Ciências/Física/Óptica",
            "Ciências"
        ));
        assert!(category_search_matches("Ciências/Física/Óptica", ""));
        assert!(!category_search_matches(
            "Ciências/Física/Óptica",
            "Química"
        ));
    }
    #[test]
    fn validator_returns_latest_revision_and_removes_fixed_errors() {
        let worker = Validator::new(eframe::egui::Context::default());
        let mut bank = crate::model::examples();
        bank.questions.truncate(1);
        let original_answer = bank.questions[0].answers[1].text.clone();
        bank.questions[0].answers[1].text = bank.questions[0].answers[0].text.clone();
        let first = worker.submit(&bank);
        let result = worker
            .results
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        assert_eq!(result.generation, first);
        assert!(result
            .error
            .as_ref()
            .unwrap()
            .contains("Alternativas a e b"));
        let p =
            crate::model::example_preview(&bank.questions[0], &result.example.unwrap()).unwrap();
        assert_eq!(p.answers[0].text, p.answers[1].text);
        // Exercise a superseded request, then a correction using the same id.
        worker.submit(&bank);
        bank.questions[0].answers[1].text = original_answer;
        let latest = worker.submit(&bank);
        loop {
            let result = worker
                .results
                .recv_timeout(std::time::Duration::from_secs(10))
                .unwrap();
            if result.generation == latest {
                assert!(result.error.is_none());
                assert!(result.example.is_none());
                break;
            }
        }
    }
}
