use crate::*;

pub(crate) fn read_import(paths: Vec<PathBuf>, remaining: usize) -> Result<ImportReview, String> {
    let mut bank = Bank {
        version: 1,
        questions: vec![],
        category_labels: vec![],
    };
    let source = paths
        .iter()
        .map(|p| p.file_name().unwrap_or_default().to_string_lossy())
        .collect::<Vec<_>>()
        .join(", ");
    for path in paths {
        let incoming = if path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("xml"))
        {
            import_xml(&path)
        } else {
            read_bank(&path)
        }
        .map_err(|e| {
            format!(
                "{}: {e}. Nenhum arquivo foi importado.",
                path.file_name().unwrap_or_default().to_string_lossy()
            )
        })?;
        merge_import(&mut bank, incoming, &BTreeSet::new(), true)?;
        if bank.questions.len() > remaining {
            return Err("A importação excederia o limite de 10.000 questões.".into());
        }
    }
    let errors = bank
        .questions
        .iter()
        .filter_map(|q| validate(q).err().map(|e| (q.id, e)))
        .collect();
    let destinations = bank
        .questions
        .iter()
        .map(|q| (q.category.clone(), q.category.clone()))
        .collect();
    Ok(ImportReview {
        bank,
        errors,
        source,
        destinations,
        overrides: BTreeMap::new(),
    })
}

fn remap_categories(review: &mut ImportReview) {
    for q in &mut review.bank.questions {
        if let Some(destination) = review
            .overrides
            .get(&q.id)
            .or_else(|| review.destinations.get(&q.category))
        {
            q.category = destination.trim().into();
        }
        if q.category.is_empty() {
            q.category = "Importadas".into();
        }
    }
    review.bank.category_labels = review
        .bank
        .category_labels
        .iter()
        .map(|p| review.destinations.get(p).unwrap_or(p).trim().to_owned())
        .filter(|p| !p.is_empty())
        .collect();

    // Category corrections can resolve validation errors from an imported file.
    review.errors = review
        .bank
        .questions
        .iter()
        .filter_map(|q| validate(q).err().map(|e| (q.id, e)))
        .collect();
}

impl App {
    pub(crate) fn import_dialog(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.import_loading {
            match rx.try_recv() {
                Ok(result) => {
                    match result {
                        Ok(review) => self.import_review = Some(review),
                        Err(e) => self.notify(e, true),
                    }
                    self.import_loading = None;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.import_loading = None;
                    self.notify(
                        "Não foi possível conferir a importação. Tente novamente.",
                        true,
                    );
                }
                _ => {}
            }
        }
        if self.import_loading.is_some() {
            egui::Modal::new(egui::Id::new("import_loading")).show(ctx, |ui| {
                ui.set_width(380.);
                ui.heading("Conferindo questões");
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Validando fórmulas, imagens e categorias...");
                });
                if ui.button("Cancelar").clicked() {
                    self.import_loading = None;
                }
            });
            return;
        }
        let Some(review) = &mut self.import_review else {
            return;
        };
        let mut confirm = false;
        let mut cancel = false;
        egui::Modal::new(egui::Id::new("import_review")).show(ctx, |ui| {
            ui.set_width(640.);
            egui::ScrollArea::vertical().id_salt("import_dialog_body").max_height((ctx.screen_rect().height()-100.).max(300.)).show(ui,|ui| {
            ui.heading("Conferir importação");
            small(ui, &review.source);
            ui.add_space(12.);
            ui.columns(3, |cols| {
                for (i, (number, label, color)) in [
                    (review.bank.questions.len(), "Questões nos arquivos", INK),
                    (review.errors.len(), "Questões com erro", ERROR),
                    (categories(&review.bank).len(), "Categorias e subcategorias", TEAL),
                ].into_iter().enumerate() {
                    egui::Frame::new().fill(BG).corner_radius(8).inner_margin(12).show(&mut cols[i], |ui| {
                        ui.label(R::new(number.to_string()).size(27.).strong().color(color));
                        small(ui, label);
                    });
                }
            });
            rule(ui);
            ui.label(R::new("Distribuição por categoria").strong());
            small(ui, "Altere o destino de cada categoria antes de importar. Também é possível ajustar cada questão abaixo.");
            let import_counts = bank_view::category_counts(&review.bank);
            let mut counts: BTreeMap<String, (usize, usize)> = categories(&review.bank).into_iter().map(|c| (c, (0, 0))).collect();
            for q in &review.bank.questions {
                let c = counts.entry(q.category.clone()).or_default();
                c.0 += 1;
                c.1 += usize::from(review.errors.contains_key(&q.id));
            }
            egui::ScrollArea::vertical().id_salt("import_categories").max_height(180.).show(ui, |ui| {
                for (path, (count, errors)) in counts {
                    ui.horizontal(|ui| {
                        ui.label(if path.is_empty() { "Sem categoria (revisar)".into() } else { format!("{} ({})",path.replace('/', " / "),import_counts.get(&path).copied().unwrap_or(0)) });
                        small(ui, format!("{count} questões · {errors} com erro"));
                    });
                    ui.push_id(&path, |ui| {
                        let destination=review.destinations.entry(path.clone()).or_insert(path.clone());
                        let choices=categories(&review.bank).into_iter().chain(self.index.categories.clone()).collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
                        category_dropdown(ui,"import_destination",destination,&choices,false,&self.index.category_counts);
                        ui.add(egui::TextEdit::singleline(destination).hint_text("Ou digite uma nova categoria / subcategoria").desired_width(f32::INFINITY));
                    });
                }
            });
            egui::CollapsingHeader::new("Ajustar categoria por questão").show(ui, |ui| {
                egui::ScrollArea::vertical().id_salt("import_question_categories").max_height(200.).show(ui, |ui| {
                    for q in &review.bank.questions {
                        ui.push_id(q.id, |ui| {
                            ui.label(&q.title);
                            let mut destination=review.overrides.get(&q.id).or_else(||review.destinations.get(&q.category)).unwrap_or(&q.category).clone();
                            if ui.add(egui::TextEdit::singleline(&mut destination).hint_text("Categoria / subcategoria").desired_width(f32::INFINITY)).changed() { review.overrides.insert(q.id,destination); }
                        });
                    }
                });
            });
            if !review.errors.is_empty() {
                rule(ui);
                ui.checkbox(&mut self.import_errors, "Incluir questões com erro para corrigir no editor");
                egui::CollapsingHeader::new("Ver problemas encontrados").show(ui, |ui| {
                    egui::ScrollArea::vertical().id_salt("import_problems").max_height(140.).show(ui, |ui| {
                        for q in &review.bank.questions {
                            if let Some(error) = review.errors.get(&q.id) {
                                ui.label(R::new(&q.title).strong());
                                ui.colored_label(ERROR, error);
                            }
                        }
                    });
                });
            }
            let count = review.bank.questions.len() - if self.import_errors { 0 } else { review.errors.len() };
            rule(ui);
            small(ui, "As categorias existentes serão reutilizadas e as novas serão criadas. As questões serão adicionadas ao banco.");
            ui.horizontal(|ui| {
                ui.add_enabled_ui(count > 0 || !review.bank.category_labels.is_empty(), |ui| {
                    confirm = primary(ui, &format!("Importar {count} questões")).clicked();
                });
                cancel = secondary(ui, "Cancelar").clicked();
            });
            });
        });
        if confirm {
            let mut review = self.import_review.take().unwrap();
            remap_categories(&mut review);
            let previous = self.bank.clone();
            match merge_import(
                &mut self.bank,
                review.bank,
                &review.errors.keys().copied().collect(),
                self.import_errors,
            ) {
                Ok(count) => {
                    if self.persist(&previous) {
                        self.sync_plan();
                        self.category = "Todas".into();
                        self.search.clear();
                        self.question_filter = bank_view::QuestionFilter::All;
                        self.notify(
                            format!(
                                "{count} questões importadas e salvas nas categorias escolhidas."
                            ),
                            false,
                        );
                    } else {
                        self.bank = previous;
                    }
                }
                Err(e) => self.notify(e, true),
            }
        } else if cancel || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.import_review = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn imported_subcategories_are_selectable_from_the_parent_and_generate_exams() {
        let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("checks/import-exam-044");
        std::fs::create_dir_all(&folder).unwrap();
        let source = folder.join("questions.xml");
        std::fs::write(&source, r#"<quiz><question type="category"><category><text>$course$/Importadas/Subcategoria</text></category></question><question type="multichoice"><name><text>Importada</text></name><questiontext><text>Escolha</text></questiontext><answer fraction="100"><text>A</text></answer><answer fraction="0"><text>B</text></answer></question></quiz>"#).unwrap();
        let review = read_import(vec![source], 100).unwrap();
        assert!(review.errors.is_empty());
        let mut bank = Bank {
            version: 1,
            questions: vec![],
            category_labels: vec![],
        };
        merge_import(&mut bank, review.bank, &BTreeSet::new(), true).unwrap();
        let mut index = bank_view::BankIndex::new(&bank);
        let validator = bank_view::Validator::new(egui::Context::default());
        let generation = validator.submit(&bank);
        let result = validator
            .results
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        assert_eq!(result.generation, generation);
        assert!(result.error.is_none());
        let question = &bank.questions[index.positions[&result.id]];
        assert_eq!(question.status, QuestionStatus::Ready);
        *index
            .ready_counts
            .entry(question.category.clone())
            .or_default() += 1;
        assert_eq!(index.category_counts["Importadas"], 1);
        assert_eq!(index.available_by_category(&[])["Importadas"], 1);
        assert!(!index.available_by_category(&[]).contains_key("Importada"));
        let mut plan = vec![];
        index.add_category_tree("Importadas", &mut plan);
        index.add_category_tree("Importadas/Subcategoria", &mut plan);
        assert_eq!(plan, [("Importadas/Subcategoria".into(), 1)]);
        assert!(!index
            .available_by_category(&plan)
            .contains_key("Importadas"));
        let batch = generate(&bank, &plan, "Ana", 1, "Prova", "", "", "", "").unwrap();
        assert_eq!(batch.students[0].questions.len(), 1);
    }
    #[test]
    fn multiple_files_remap_ids_and_categories_without_partial_import() {
        let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("checks/multi-import-042");
        std::fs::create_dir_all(&folder).unwrap();
        let first = folder.join("first.json");
        let second = folder.join("second.xml");
        let mut bank = examples();
        bank.questions.truncate(1);
        bank.questions[0].status = QuestionStatus::Draft;
        save_bank(&first, &bank).unwrap();
        std::fs::write(&second,r#"<quiz><question type="multichoice"><name><text>Segunda</text></name><questiontext><text>Escolha</text></questiontext><answer fraction="100"><text>A</text></answer><answer fraction="0"><text>B</text></answer></question></quiz>"#).unwrap();
        let mut review = read_import(vec![first.clone(), second.clone()], 10).unwrap();
        assert_eq!(review.bank.questions.len(), 2);
        assert_ne!(review.bank.questions[0].id, review.bank.questions[1].id);
        assert_eq!(review.bank.questions[0].status, QuestionStatus::Draft);
        review
            .destinations
            .insert("Mecânica".into(), "Nova/Sub".into());
        review
            .destinations
            .insert("Importadas".into(), "Destino".into());
        review
            .overrides
            .insert(review.bank.questions[1].id, "Individual".into());
        remap_categories(&mut review);
        assert_eq!(review.bank.questions[0].category, "Nova/Sub");
        assert_eq!(review.bank.questions[1].category, "Individual");
        assert!(review.errors.is_empty());
        assert!(read_import(vec![first.clone(), second.clone()], 1).is_err());
        std::fs::write(&second, "<quiz><question>").unwrap();
        assert!(read_import(vec![first, second], 10).is_err());
    }
}
