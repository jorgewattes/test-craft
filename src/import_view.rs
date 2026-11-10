use crate::*;

impl App {
    pub(crate) fn import_dialog(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.import_loading {
            match rx.try_recv() {
                Ok(review) => {
                    self.import_review = Some(review);
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
        let Some(review) = &self.import_review else {
            return;
        };
        let mut confirm = false;
        let mut cancel = false;
        egui::Modal::new(egui::Id::new("import_review")).show(ctx, |ui| {
            ui.set_width(640.);
            ui.heading("Conferir importação");
            small(ui, &review.source);
            ui.add_space(12.);
            ui.columns(3, |cols| {
                for (i, (number, label, color)) in [
                    (review.bank.questions.len(), "Questões no arquivo", INK),
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
            small(ui, "Cada questão será alocada na categoria indicada no arquivo.");
            let mut counts: BTreeMap<String, (usize, usize)> = categories(&review.bank).into_iter().map(|c| (c, (0, 0))).collect();
            for q in &review.bank.questions {
                let c = counts.entry(q.category.clone()).or_default();
                c.0 += 1;
                c.1 += usize::from(review.errors.contains_key(&q.id));
            }
            egui::ScrollArea::vertical().id_salt("import_categories").max_height(180.).show(ui, |ui| {
                for (path, (count, errors)) in counts {
                    ui.horizontal(|ui| {
                        ui.label(if path.is_empty() { "Sem categoria (revisar)".into() } else { path.replace('/', " / ") });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            small(ui, format!("{count} questões · {errors} com erro"));
                        });
                    });
                }
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
        if confirm {
            let review = self.import_review.take().unwrap();
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
                        self.notify(
                            format!(
                                "{count} questões importadas e salvas nas categorias do arquivo."
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
