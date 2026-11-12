use crate::*;

pub(crate) fn proof_category_picker(
    ctx: &egui::Context,
    search: &mut String,
    focus: &mut bool,
    index: &bank_view::BankIndex,
    plan: &[(String, usize)],
    validating: bool,
) -> (Option<String>, bool) {
    let mut selected = None;
    let mut close = false;
    let available = index.available_by_category(plan);
    let totals = index.available_by_category(&[]);
    let response = egui::Modal::new(egui::Id::new("proof_category_picker")).show(ctx, |ui| {
        ui.set_width(460.0_f32.min(ctx.screen_rect().width() - 70.));
        ui.heading("Adicionar categoria à prova");
        let input = ui.add(egui::TextEdit::singleline(search)
            .id_salt("proof_category_search")
            .hint_text("Buscar categoria ou subcategoria")
            .desired_width(f32::INFINITY));
        if *focus {
            input.request_focus();
            *focus = false;
        }
        small(ui, "Expanda uma categoria para escolher suas subcategorias ou níveis. Só aparecem níveis com questões prontas.");
        if validating { small(ui, "Validando questões; as quantidades serão atualizadas."); }
        ui.add_space(8.);
        egui::ScrollArea::vertical().id_salt("proof_category_results")
            .max_height((ctx.screen_rect().height() - 250.).clamp(100., 440.))
            .show(ui, |ui| {
                if !index.categories.iter().any(|cat| proof_search_matches(cat, &totals, search)) {
                    small(ui, "Nenhuma categoria encontrada. Ajuste a busca ou crie uma categoria no banco.");
                } else {
                    category_tree(ui, "", &index.categories, &available, &totals, search, &mut selected);
                }
            });
        close = ui.button("Fechar").clicked();
    });
    (selected, close || response.should_close())
}

fn proof_search_matches(category: &str, totals: &BTreeMap<String, usize>, search: &str) -> bool {
    bank_view::category_search_matches(category, search)
        || proof_levels().any(|level| {
            let key = proof_pool_key(category, level);
            totals.get(&key).copied().unwrap_or(0) > 0
                && bank_view::category_search_matches(&proof_pool_label(&key), search)
        })
}
fn proof_levels() -> impl Iterator<Item = Option<QuestionDifficulty>> {
    QuestionDifficulty::ALL.into_iter().map(Some).chain(std::iter::once(None))
}

fn category_tree(
    ui: &mut egui::Ui,
    parent: &str,
    categories: &[String],
    available: &BTreeMap<String, usize>,
    totals: &BTreeMap<String, usize>,
    search: &str,
    selected: &mut Option<String>,
) {
    for cat in categories
        .iter()
        .filter(|cat| cat.rsplit_once('/').map_or("", |(p, _)| p) == parent)
    {
        if !categories
            .iter()
            .any(|p| category_matches(p, cat) && proof_search_matches(p, totals, search))
        {
            continue;
        }
        let count = available.get(cat).copied().unwrap_or(0);
        let leaf = cat.rsplit('/').next().unwrap_or(cat);
        let children = categories
            .iter()
            .any(|p| p.rsplit_once('/').is_some_and(|(p, _)| p == cat));
        let levels: Vec<_> = proof_levels().filter(|level| totals.get(&proof_pool_key(cat, *level)).copied().unwrap_or(0) > 0).collect();
        if children || !levels.is_empty() {
            egui::CollapsingHeader::new(R::new(format!("{leaf}   ·   {count}")).strong())
                .id_salt(("proof_tree", cat, search))
                .default_open(!search.trim().is_empty())
                .show(ui, |ui| {
                    category_add_row(ui, cat, "Todas desta categoria", count, selected);
                    for level in &levels {
                        let key = proof_pool_key(cat, *level);
                        if bank_view::category_search_matches(&proof_pool_label(&key), search) {
                            category_add_row(ui, &key, level.map_or("Sem classificação", QuestionDifficulty::label), available.get(&key).copied().unwrap_or(0), selected);
                        }
                    }
                    category_tree(ui, cat, categories, available, totals, search, selected);
                });
        } else {
            category_add_row(ui, cat, leaf, count, selected);
        }
    }
}
fn category_add_row(
    ui: &mut egui::Ui,
    path: &str,
    label: &str,
    count: usize,
    selected: &mut Option<String>,
) {
    ui.push_id(path, |ui| {
        ui.horizontal(|ui| {
            let response = ui.add_enabled(
                count > 0,
                egui::Button::new(label)
                    .wrap()
                    .min_size(Vec2::new((ui.available_width() - 75.).max(120.), 36.)),
            );
            if response.on_hover_text(proof_pool_label(path).replace('/', " / ")).clicked() {
                *selected = Some(path.into());
            }
            small(
                ui,
                if count == 0 {
                    "0 disponíveis".into()
                } else {
                    format!("{count} prontas")
                },
            );
        });
    });
}

#[cfg(test)]
mod picker_tests {
    use super::*;

    #[test]
    fn search_finds_only_existing_ready_levels_with_accent_insensitive_labels() {
        let mut bank = examples();
        bank.questions.truncate(1);
        bank.questions[0].category = "Física/Óptica".into();
        bank.questions[0].difficulty = Some(QuestionDifficulty::Easy);
        let mut index = bank_view::BankIndex::new(&bank);
        assert!(!proof_search_matches("Física/Óptica", &index.available_by_category(&[]), "facil"));
        index.record_ready(&bank.questions[0]);
        let totals = index.available_by_category(&[]);
        assert!(proof_search_matches("Física/Óptica", &totals, "optica facil"));
        assert!(proof_search_matches("Física", &totals, "facil"));
        assert!(!proof_search_matches("Física/Óptica", &totals, "dificil"));
        assert!(!proof_search_matches("Física/Óptica", &totals, "sem classificacao"));
    }

    #[test]
    fn proof_category_search_keeps_focus_and_open_state_while_typing() {
        let ctx = egui::Context::default();
        let index = bank_view::BankIndex::new(&examples());
        let mut search = String::new();
        let mut focus = true;
        for text in [None, Some("meca"), Some("nica")] {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(1120., 740.),
                )),
                events: text
                    .map(|text| vec![egui::Event::Text(text.into())])
                    .unwrap_or_default(),
                ..Default::default()
            };
            let _ = ctx.run(input, |ctx| {
                let (selected, closed) =
                    proof_category_picker(ctx, &mut search, &mut focus, &index, &[], false);
                assert!(selected.is_none());
                assert!(!closed, "Typing must not close the category picker");
            });
        }
        assert_eq!(search, "mecanica");
        assert!(bank_view::category_search_matches("Mecânica", &search));
        assert!(!focus);
    }
}

impl App {
    pub(crate) fn category_bulk_controls(&mut self, ui: &mut egui::Ui) {
        self.category_checked
            .retain(|p| self.index.categories.contains(p));
        ui.horizontal_wrapped(|ui| {
            if secondary(ui, "Selecionar visíveis").clicked() {
                self.category_checked.extend(
                    self.index
                        .categories
                        .iter()
                        .filter(|p| {
                            p.to_lowercase()
                                .contains(&self.category_search.to_lowercase())
                        })
                        .cloned(),
                );
            }
            if secondary(ui, "Limpar seleção").clicked() {
                self.category_checked.clear();
                self.category_bulk_delete = false;
            }
            small(ui, format!("{} selecionadas", self.category_checked.len()));
        });
        if self.category_checked.is_empty() {
            return;
        }
        let choices = self
            .index
            .categories
            .iter()
            .filter(|p| {
                !self
                    .category_checked
                    .iter()
                    .any(|root| category_matches(p, root))
            })
            .cloned()
            .collect::<Vec<_>>();
        small(ui, "Mover categorias selecionadas para dentro de:");
        category_dropdown(
            ui,
            "bulk_category_destination",
            &mut self.category_bulk_destination,
            &choices,
            false,
            &self.index.category_counts,
        );
        let mut operation = None;
        ui.horizontal_wrapped(|ui| {
            ui.add_enabled_ui(choices.contains(&self.category_bulk_destination), |ui| {
                if secondary(ui, "Mover selecionadas").clicked() {
                    operation = Some(Some(self.category_bulk_destination.clone()));
                }
            });
            if secondary(ui, "Apagar selecionadas…").clicked() {
                self.category_bulk_delete = true;
            }
        });
        if self.category_bulk_delete {
            let count = self
                .bank
                .questions
                .iter()
                .filter(|q| {
                    self.category_checked
                        .iter()
                        .any(|p| category_matches(&q.category, p))
                })
                .count();
            ui.colored_label(ERROR, format!("Apagar {} categorias selecionadas, suas subcategorias e {count} questões? Esta ação não pode ser desfeita.",self.category_checked.len()));
            ui.horizontal(|ui| {
                if secondary(ui, "Confirmar exclusão em lote").clicked() {
                    operation = Some(None);
                }
                if secondary(ui, "Cancelar exclusão").clicked() {
                    self.category_bulk_delete = false;
                }
            });
        }
        if let Some(destination) = operation {
            let previous = self.bank.clone();
            match bulk_categories(
                &mut self.bank,
                &self.category_checked,
                destination.as_deref(),
            ) {
                Ok(()) => {
                    if self.persist(&previous) {
                        self.category_checked.clear();
                        self.category_bulk_delete = false;
                        self.category_edit = None;
                        self.category_name.clear();
                        self.category_parent.clear();
                        self.category = "Todas".into();
                        self.category_error = None;
                        self.sync_plan();
                        self.notify("Categorias atualizadas e salvas.", false);
                    } else {
                        self.bank = previous;
                        self.category_error = Some(self.status.clone());
                    }
                }
                Err(e) => self.category_error = Some(e),
            }
        }
    }
    pub(crate) fn category_delete_dialog(&mut self, ctx: &egui::Context) {
        let Some(path) = self.category_delete.clone() else {
            return;
        };
        let count = self.index.category_counts.get(&path).copied().unwrap_or(0);
        let mut confirm = false;
        let mut cancel = false;
        egui::Modal::new(egui::Id::new("remove_category")).show(ctx,|ui| {
            ui.set_width(480.);
            ui.heading("Excluir categoria");
            ui.label(R::new(path.replace('/'," / ")).strong().color(TEAL));
            rule(ui);
            ui.label(format!("A categoria e suas subcategorias serão excluídas. {count} questões serão preservadas."));
            if count > 0 {
                small(ui,"Escolha para onde mover todas as questões antes de excluir.");
                let choices=self.index.categories.iter().filter(|p|!category_matches(p,&path)).cloned().collect::<Vec<_>>();
                category_dropdown(ui,"delete_category_destination",&mut self.category_delete_destination,&choices,false,&self.index.category_counts);
                if choices.is_empty(){ui.colored_label(ERROR,"Crie uma categoria de destino antes de excluir esta categoria.");}
            }
            if let Some(error)=&self.category_error{ui.colored_label(ERROR,error);}
            rule(ui);
            ui.horizontal(|ui| {
                ui.add_enabled_ui(count==0 || !self.category_delete_destination.is_empty(),|ui| {
                    confirm=primary(ui,if count==0{"Confirmar exclusão"}else{"Mover e excluir"}).clicked();
                });
                cancel=secondary(ui,"Cancelar").clicked();
            });
        });
        if confirm {
            let previous = self.bank.clone();
            match remove_category(&mut self.bank, &path, &self.category_delete_destination) {
                Ok(()) => {
                    if self.persist(&previous) {
                        if category_matches(&self.draft.category, &path) {
                            self.draft.category = self.category_delete_destination.clone();
                            self.validation = None;
                        }
                        self.sync_plan();
                        self.category = "Todas".into();
                        self.category_edit = None;
                        self.category_name.clear();
                        self.category_parent.clear();
                        self.category_delete = None;
                        self.category_dialog = true;
                        self.category_error = None;
                        self.notify(
                            format!("Categoria excluída. {count} questões preservadas."),
                            false,
                        );
                    } else {
                        self.bank = previous;
                        self.category_error = Some(self.status.clone());
                    }
                }
                Err(e) => self.category_error = Some(e),
            }
        } else if cancel || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.category_delete = None;
            self.category_dialog = true;
            self.category_error = None;
        }
    }
}
