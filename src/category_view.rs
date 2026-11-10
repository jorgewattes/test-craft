use crate::*;

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
                category_dropdown(ui,"delete_category_destination",&mut self.category_delete_destination,&choices,false);
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
