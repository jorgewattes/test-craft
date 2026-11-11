use crate::*;

#[derive(Clone, Default, Serialize, Deserialize)]
pub(crate) struct StudentGrid {
    rows: Vec<Vec<String>>,
    name_column: usize,
    skip_header: bool,
    #[serde(skip)]
    error: Option<String>,
}

impl StudentGrid {
    pub fn from_names(names: &str) -> Self {
        Self {
            rows: names.lines().map(|n| vec![n.to_owned()]).collect(),
            ..Default::default()
        }
    }
    fn columns(&self) -> usize {
        self.rows.iter().map(Vec::len).max().unwrap_or(1).max(1)
    }
    pub fn names(&self) -> String {
        self.rows
            .iter()
            .skip(usize::from(self.skip_header))
            .filter_map(|r| r.get(self.name_column))
            .map(|n| n.trim())
            .filter(|n| !n.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
    fn paste(&mut self, row: usize, column: usize, text: &str) -> Result<(), String> {
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        let cells: Vec<Vec<String>> = normalized
            .trim_end_matches('\n')
            .split('\n')
            .map(|line| line.split('\t').map(str::to_owned).collect())
            .collect();
        let width = cells.iter().map(Vec::len).max().unwrap_or(1);
        if row + cells.len() > 1001 || column + width > 32 {
            return Err("Cole até 1.000 alunos e um cabeçalho, com até 32 colunas. A tabela foi preservada.".into());
        }
        self.rows
            .resize_with(self.rows.len().max(row + cells.len()), Vec::new);
        for (offset, values) in cells.into_iter().enumerate() {
            let target = &mut self.rows[row + offset];
            target.resize(target.len().max(column + values.len()), String::new());
            for (c, value) in values.into_iter().enumerate() {
                target[column + c] = value;
            }
        }
        Ok(())
    }
    fn remove_column(&mut self, column: usize) {
        for row in &mut self.rows {
            if column < row.len() {
                row.remove(column);
            }
        }
        if self.name_column > column {
            self.name_column -= 1;
        }
        self.name_column = self.name_column.min(self.columns() - 1);
    }
    pub fn show(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        let columns = self.columns();
        self.name_column = self.name_column.min(columns - 1);
        // Keep a blank row for manual entry and pasting another list.
        if self.rows.is_empty()
            || self
                .rows
                .last()
                .is_some_and(|r| r.iter().any(|s| !s.is_empty()))
        {
            if self.rows.len() < 1001 {
                self.rows.push(vec![String::new(); columns]);
            }
        }
        small(ui, "Cole do Acadêmico, Excel ou outro site em uma célula. Escolha a coluna dos nomes e remova as extras.");
        let mut remove_column = None;
        let mut remove_row = None;
        let mut pasted = None;
        egui::ScrollArea::horizontal()
            .id_salt("students_horizontal")
            .show(ui, |ui| {
                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(1., 1.);
                    ui.spacing_mut().interact_size.y = 26.;
                    ui.spacing_mut().button_padding = Vec2::new(5., 3.);
                    let visuals = ui.visuals_mut();
                    for widget in [
                        &mut visuals.widgets.inactive,
                        &mut visuals.widgets.hovered,
                        &mut visuals.widgets.active,
                    ] {
                        widget.corner_radius = egui::CornerRadius::ZERO;
                    }
                    egui::Grid::new("student_headers")
                        .spacing([1., 1.])
                        .show(ui, |ui| {
                            ui.add_sized([34., 26.], egui::Label::new("Linha"));
                            for c in 0..columns {
                                ui.allocate_ui_with_layout(
                                    Vec2::new(170., 26.),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        changed |= ui
                                            .radio_value(
                                                &mut self.name_column,
                                                c,
                                                format!("{} · Nomes", column_label(c)),
                                            )
                                            .changed();
                                        if ui
                                            .small_button("×")
                                            .on_hover_text("Remover esta coluna")
                                            .clicked()
                                        {
                                            remove_column = Some(c);
                                        }
                                    },
                                );
                            }
                            ui.end_row();
                        });
                    egui::ScrollArea::vertical()
                        .id_salt("students_vertical")
                        .max_height(210.)
                        .show_rows(ui, 28., self.rows.len(), |ui, range| {
                            egui::Grid::new("student_cells")
                                .spacing([1., 1.])
                                .show(ui, |ui| {
                                    for r in range {
                                        ui.add_sized(
                                            [34., 28.],
                                            egui::Label::new((r + 1).to_string()),
                                        );
                                        self.rows[r].resize(columns, String::new());
                                        for c in 0..columns {
                                            let id = egui::Id::new(("student_cell", r, c));
                                            if ui.memory(|m| m.has_focus(id)) {
                                                ui.input_mut(|i| {
                                                    i.events.retain(|event| {
                                                        if let egui::Event::Paste(text) = event {
                                                            if text.contains(['\t', '\n', '\r']) {
                                                                pasted = Some((r, c, text.clone()));
                                                                return false;
                                                            }
                                                        }
                                                        true
                                                    })
                                                });
                                            }
                                            let color = if self.skip_header && r == 0 {
                                                LINE
                                            } else if c == self.name_column {
                                                PALE
                                            } else {
                                                C::WHITE
                                            };
                                            changed |= ui
                                                .add_sized(
                                                    [170., 28.],
                                                    egui::TextEdit::singleline(
                                                        &mut self.rows[r][c],
                                                    )
                                                    .id(id)
                                                    .background_color(color)
                                                    .margin(Vec2::new(5., 5.)),
                                                )
                                                .changed();
                                        }
                                        if ui
                                            .small_button("×")
                                            .on_hover_text("Remover esta linha")
                                            .clicked()
                                        {
                                            remove_row = Some(r);
                                        }
                                        ui.end_row();
                                    }
                                });
                        });
                });
            });
        if let Some((r, c, text)) = pasted {
            match self.paste(r, c, &text) {
                Ok(()) => {
                    changed = true;
                    self.error = None;
                }
                Err(e) => self.error = Some(e),
            }
        }
        if let Some(c) = remove_column {
            self.remove_column(c);
            changed = true;
        }
        if let Some(r) = remove_row {
            self.rows.remove(r);
            changed = true;
        }
        ui.horizontal_wrapped(|ui| {
            changed |= ui
                .checkbox(&mut self.skip_header, "Primeira linha é cabeçalho")
                .changed();
            if ui
                .add_enabled(columns < 32, egui::Button::new("+ Coluna"))
                .clicked()
            {
                for row in &mut self.rows {
                    row.resize(columns + 1, String::new());
                }
                changed = true;
            }
            if ui.button("Remover linhas vazias").clicked() {
                self.rows.retain(|r| r.iter().any(|s| !s.trim().is_empty()));
                changed = true;
            }
        });
        if let Some(error) = &self.error {
            ui.colored_label(ERROR, error);
        }
        small(
            ui,
            format!(
                "{} alunos · coluna {} usada na prova",
                self.names().lines().count(),
                column_label(self.name_column)
            ),
        );
        changed
    }
}

fn column_label(index: usize) -> String {
    if index < 26 {
        ((b'A' + index as u8) as char).to_string()
    } else {
        format!("A{}", (b'A' + (index - 26) as u8) as char)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn focused_cell_receives_a_table_paste_instead_of_flattening_it() {
        let ctx = egui::Context::default();
        let mut grid = StudentGrid::default();
        for events in [
            vec![],
            vec![egui::Event::Paste("12\tAna\n13\tBruno".into())],
        ] {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(800., 600.),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    ctx.memory_mut(|m| {
                        m.request_focus(egui::Id::new(("student_cell", 0usize, 0usize)))
                    });
                    egui::CentralPanel::default().show(ctx, |ui| {
                        grid.show(ui);
                    });
                },
            );
        }
        grid.name_column = 1;
        assert_eq!(grid.names(), "Ana\nBruno");
        assert_eq!(grid.rows[0], ["12", "Ana"]);
        assert_eq!(grid.rows[1], ["13", "Bruno"]);
    }
    #[test]
    fn academic_table_paste_edit_and_reload() {
        let mut grid = StudentGrid::default();
        grid.paste(
            0,
            0,
            "Matrícula\tNome\tCurso\r\n12\tAna Souza\tFísica\r\n13\tBruno Lima\tFísica\r\n",
        )
        .unwrap();
        grid.name_column = 1;
        grid.skip_header = true;
        assert_eq!(grid.names(), "Ana Souza\nBruno Lima");
        grid.remove_column(0);
        grid.remove_column(1);
        assert_eq!(grid.name_column, 0);
        grid.paste(3, 0, "Carla\nDiego").unwrap();
        let restored: StudentGrid =
            serde_json::from_str(&serde_json::to_string(&grid).unwrap()).unwrap();
        assert_eq!(restored.names(), "Ana Souza\nBruno Lima\nCarla\nDiego");
        assert_eq!(
            StudentGrid::from_names("Antigo\nNome").names(),
            "Antigo\nNome"
        );
    }
    #[test]
    fn paste_keeps_blank_cells_and_rejects_oversized_tables_atomically() {
        let mut grid = StudentGrid::from_names("Ana");
        grid.paste(0, 1, "\tX\nY\t").unwrap();
        assert_eq!(grid.rows[0], ["Ana", "", "X"]);
        assert_eq!(grid.rows[1], ["", "Y", ""]);
        let before = grid.rows.clone();
        assert!(grid.paste(1000, 0, "a\nb").is_err());
        assert!(grid.paste(0, 32, "a").is_err());
        assert_eq!(grid.rows, before);
    }
}
