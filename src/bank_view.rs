use crate::model::{
    categories, category_matches, proof_pool_key, proof_pool_parts, validate_with_example, Bank, Question, QuestionDifficulty,
    QuestionStatus, ValidationExample,
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

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum DifficultyFilter {
    #[default]
    All,
    Unclassified,
    Level(QuestionDifficulty),
}
impl DifficultyFilter {
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "Todos os níveis",
            Self::Unclassified => "Sem classificação",
            Self::Level(level) => level.label(),
        }
    }
    fn matches(self, question: &Question) -> bool {
        match self {
            Self::All => true,
            Self::Unclassified => question.difficulty.is_none(),
            Self::Level(level) => question.difficulty == Some(level),
        }
    }
}

/// Lightweight lookup/search data; never clone images or rebuild text on repaint.
pub struct BankIndex {
    pub positions: BTreeMap<u64, usize>,
    pub categories: Vec<String>,
    pub category_counts: BTreeMap<String, usize>,
    pub ready_counts: BTreeMap<String, usize>,
    pub ready_pool_counts: BTreeMap<String, usize>,
    search: Vec<String>,
    filter: Option<(String, String, QuestionFilter, DifficultyFilter)>,
    matches: Arc<Vec<usize>>,
}
impl BankIndex {
    pub fn new(bank: &Bank) -> Self {
        Self {
            category_counts: category_counts(bank),
            ready_counts: BTreeMap::new(),
            ready_pool_counts: BTreeMap::new(),
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
        let mut counts = BTreeMap::new();
        for (pool, count) in &self.ready_pool_counts {
            let (category, level) = proof_pool_parts(pool);
            if plan.iter().any(|(key, _)| key == category || key == pool) {
                continue;
            }
            let mut path = category;
            loop {
                *counts.entry(path.to_owned()).or_default() += count;
                *counts.entry(proof_pool_key(path, level.unwrap())).or_default() += count;
                match path.rsplit_once('/') {
                    Some((parent, _)) => path = parent,
                    None => break,
                }
            }
        }
        counts
    }
    pub fn record_ready(&mut self, question: &Question) {
        *self.ready_counts.entry(question.category.clone()).or_default() += 1;
        *self.ready_pool_counts.entry(proof_pool_key(&question.category, question.difficulty)).or_default() += 1;
    }
    pub fn plan_count(&self, key: &str) -> usize {
        if proof_pool_parts(key).1.is_some() {
            self.ready_pool_counts.get(key).copied().unwrap_or(0)
        } else {
            self.ready_counts.get(key).copied().unwrap_or(0)
        }
    }
    /// Expand a parent into disjoint direct-category pools. Existing plan rows
    /// keep their quantities and no question can be sampled twice via a parent.
    pub fn add_category_tree(&self, root: &str, plan: &mut Vec<(String, usize)>) {
        let (root, level) = proof_pool_parts(root);
        for path in &self.categories {
            if category_matches(path, root)
                && self.ready_counts.get(path).copied().unwrap_or(0) > 0
                && !plan.iter().any(|(key, _)| key == path)
            {
                let partial = plan.iter().any(|(key, _)| proof_pool_parts(key).0 == path);
                if level.is_none() && !partial {
                    plan.push((path.clone(), 1));
                } else {
                    for (pool, count) in &self.ready_pool_counts {
                        let (category, difficulty) = proof_pool_parts(pool);
                        if category == path && *count > 0 && (level.is_none() || level == difficulty)
                            && !plan.iter().any(|(key, _)| key == pool) {
                            plan.push((pool.clone(), 1));
                        }
                    }
                }
            }
        }
    }
    pub fn filter(
        &mut self,
        bank: &Bank,
        category: &str,
        search: &str,
        status: QuestionFilter,
        difficulty: DifficultyFilter,
        errors: &BTreeMap<u64, String>,
        pending: &std::collections::BTreeSet<u64>,
    ) -> (Arc<Vec<usize>>, bool) {
        let key = (
            category.to_owned(),
            search.trim().to_lowercase(),
            status,
            difficulty,
        );
        let changed = self.filter.as_ref() != Some(&key);
        if changed {
            self.matches = Arc::new(
                bank.questions
                    .iter()
                    .enumerate()
                    .filter(|(i, q)| {
                        (category == "Todas" || category_matches(&q.category, category))
                            && self.search[*i].contains(&key.1)
                            && difficulty.matches(q)
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
    fn level_selection_expands_descendants_without_overlapping_existing_rows() {
        use crate::model::{generate_with_mode, proof_pool_matches, QuestionDifficulty::*};
        let template = crate::model::examples().questions.remove(0);
        let mut bank = Bank { version: 1, questions: vec![], category_labels: vec![] };
        for (i, (category, level)) in [
            ("Física", Some(Easy)),
            ("Física/Óptica", Some(Easy)),
            ("Física/Óptica", Some(Hard)),
            ("Física/Óptica", None),
            ("Física/Óptica/Lentes", Some(Easy)),
            ("Química", Some(Easy)),
        ].into_iter().enumerate() {
            let mut q = template.clone();
            q.id = i as u64 + 1;
            q.title = format!("Questão {i}");
            q.category = category.into();
            q.difficulty = level;
            bank.questions.push(q);
        }
        let mut index = BankIndex::new(&bank);
        for q in &bank.questions { index.record_ready(q); }
        let easy = proof_pool_key("Física", Some(Easy));
        assert_eq!(index.available_by_category(&[])[&easy], 3);
        let mut plan = vec![];
        index.add_category_tree(&easy, &mut plan);
        assert_eq!(plan.len(), 3);
        assert!(plan.iter().all(|(key, _)| index.plan_count(key) == 1));
        let unchanged = plan.clone();
        index.add_category_tree(&easy, &mut plan);
        assert_eq!(plan, unchanged);
        assert!(!index.available_by_category(&plan).contains_key(&easy));
        assert_eq!(index.available_by_category(&plan)["Física"], 2);
        for mode in [false, true] {
            for seed in 0..8 {
                let batch = generate_with_mode(&bank, &plan, "Ana\nBia", seed, "Prova", "", "", "", "", mode).unwrap();
                for student in batch.students {
                    assert_eq!(student.questions.len(), 3);
                    for question in student.questions {
                        let original = bank.questions.iter().find(|q| q.title == question.title).unwrap();
                        assert_eq!(original.difficulty, Some(Easy));
                        assert!(category_matches(&original.category, "Física"));
                    }
                }
            }
        }
        // Adding the whole category after one level adds only the other levels.
        index.add_category_tree("Física", &mut plan);
        assert_eq!(plan.len(), 5);
        for q in bank.questions.iter().filter(|q| category_matches(&q.category, "Física")) {
            assert_eq!(plan.iter().filter(|(key, _)| proof_pool_matches(key, q)).count(), 1);
        }
        assert!(index.available_by_category(&plan).get("Física").is_none());
    }

    #[test]
    fn difficulty_filter_combines_with_status_search_and_category_and_updates_cache() {
        let mut bank = crate::model::examples();
        let template = bank.questions[0].clone();
        bank.questions = QuestionDifficulty::ALL
            .into_iter()
            .enumerate()
            .map(|(i, level)| {
                let mut q = template.clone();
                q.id = i as u64;
                q.difficulty = Some(level);
                q
            })
            .collect();
        let mut unclassified = template.clone();
        unclassified.id = 10;
        bank.questions.push(unclassified);
        let errors = BTreeMap::from([(2, "Revisar".into())]);
        let pending = Default::default();
        let mut index = BankIndex::new(&bank);
        for (i, level) in QuestionDifficulty::ALL.into_iter().enumerate() {
            let (matches, changed) = index.filter(
                &bank,
                "Todas",
                "",
                QuestionFilter::All,
                DifficultyFilter::Level(level),
                &errors,
                &pending,
            );
            assert!(changed);
            assert_eq!(*matches, vec![i]);
        }
        assert_eq!(
            *index
                .filter(
                    &bank,
                    "Todas",
                    "",
                    QuestionFilter::All,
                    DifficultyFilter::Unclassified,
                    &errors,
                    &pending
                )
                .0,
            vec![5]
        );
        assert_eq!(
            index
                .filter(
                    &bank,
                    "Todas",
                    "",
                    QuestionFilter::All,
                    DifficultyFilter::All,
                    &errors,
                    &pending
                )
                .0
                .len(),
            6
        );
        assert_eq!(
            *index
                .filter(
                    &bank,
                    &template.category,
                    &template.title,
                    QuestionFilter::Error,
                    DifficultyFilter::Level(QuestionDifficulty::Medium),
                    &errors,
                    &pending
                )
                .0,
            vec![2]
        );
        assert!(index
            .filter(
                &bank,
                "Todas",
                "",
                QuestionFilter::Ready,
                DifficultyFilter::Level(QuestionDifficulty::Medium),
                &errors,
                &pending
            )
            .0
            .is_empty());
        assert!(index
            .filter(
                &bank,
                "Ausente",
                "",
                QuestionFilter::All,
                DifficultyFilter::Level(QuestionDifficulty::Medium),
                &errors,
                &pending
            )
            .0
            .is_empty());
        assert!(index
            .filter(
                &bank,
                "Todas",
                "sem correspondência",
                QuestionFilter::All,
                DifficultyFilter::Level(QuestionDifficulty::Medium),
                &errors,
                &pending
            )
            .0
            .is_empty());
        bank.questions[2].difficulty = Some(QuestionDifficulty::Hard);
        index = BankIndex::new(&bank);
        assert_eq!(
            *index
                .filter(
                    &bank,
                    "Todas",
                    "",
                    QuestionFilter::All,
                    DifficultyFilter::Level(QuestionDifficulty::Hard),
                    &errors,
                    &pending
                )
                .0,
            vec![2, 3]
        );
    }
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
                    DifficultyFilter::All,
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
                    DifficultyFilter::All,
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
                DifficultyFilter::All,
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
            .filter(
                &bank,
                "Todas",
                "",
                QuestionFilter::Ready,
                DifficultyFilter::All,
                &errors,
                &pending
            )
            .0
            .is_empty());
        assert_eq!(
            *index
                .filter(
                    &bank,
                    "Todas",
                    "",
                    QuestionFilter::Draft,
                    DifficultyFilter::All,
                    &errors,
                    &pending
                )
                .0,
            vec![1]
        );
        assert_eq!(
            *index
                .filter(
                    &bank,
                    "Todas",
                    "",
                    QuestionFilter::Error,
                    DifficultyFilter::All,
                    &errors,
                    &pending
                )
                .0,
            vec![2]
        );
        assert!(index
            .filter(
                &bank,
                "Ausente",
                "",
                QuestionFilter::Error,
                DifficultyFilter::All,
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
                DifficultyFilter::All,
                &errors,
                &pending
            )
            .0
            .is_empty());
        pending.clear();
        index.invalidate_filter();
        assert_eq!(
            *index
                .filter(
                    &bank,
                    "Todas",
                    "",
                    QuestionFilter::Ready,
                    DifficultyFilter::All,
                    &errors,
                    &pending
                )
                .0,
            vec![0]
        );
        errors.clear();
        index.invalidate_filter();
        assert!(index
            .filter(
                &bank,
                "Todas",
                "",
                QuestionFilter::Error,
                DifficultyFilter::All,
                &errors,
                &pending
            )
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
