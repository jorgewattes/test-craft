use crate::*;

impl App {
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
