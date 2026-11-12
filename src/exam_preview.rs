use crate::*;

#[derive(Default)]
pub(crate) struct ExamPreview {
    pub open: bool,
    document: Option<(usize, pdf::PreviewDocument)>,
    zoom: f32,
}
impl ExamPreview {
    pub fn invalidate(&mut self) {
        self.document = None;
    }
    pub fn install_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        let system = std::env::var_os("WINDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("C:/Windows"))
            .join("Fonts");
        for (name, file) in [("pdf_regular", "arial.ttf"), ("pdf_bold", "arialbd.ttf")] {
            let mut family = fonts
                .families
                .get(&egui::FontFamily::Proportional)
                .cloned()
                .unwrap_or_default();
            if let Ok(bytes) = std::fs::read(system.join(file)) {
                fonts
                    .font_data
                    .insert(name.into(), egui::FontData::from_owned(bytes).into());
                family.insert(0, name.into());
            }
            fonts
                .families
                .insert(egui::FontFamily::Name(name.into()), family);
        }
        ctx.set_fonts(fonts);
    }
    pub fn show(&mut self, ctx: &egui::Context, batch: &mut Batch, student: &mut usize) -> bool {
        if !self.open || batch.students.is_empty() {
            return false;
        }
        *student = (*student).min(batch.students.len() - 1);
        if self.zoom == 0. {
            self.zoom = 1.;
        }
        let mut open = true;
        let mut moved = false;
        egui::Window::new("Prévia da prova · PDF")
            .id(egui::Id::new("exam_pdf_popup"))
            .open(&mut open).collapsible(false).resizable(true)
            .default_size(Vec2::new(960., 760.).min(ctx.screen_rect().size() - Vec2::splat(60.)))
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    small(ui, "Aluno(a)");
                    egui::ComboBox::from_id_salt("pdf_student").width(240.)
                        .selected_text(&batch.students[*student].name).show_ui(ui, |ui| {
                            for (i, s) in batch.students.iter().enumerate() { ui.selectable_value(student, i, &s.name); }
                        });
                    small(ui, "Zoom");
                    ui.add(egui::Slider::new(&mut self.zoom, 0.6..=1.8).show_value(false).step_by(0.1));
                    small(ui, format!("{:.0}%", self.zoom * 100.));
                    if ui.button("Ajustar à largura").clicked() { self.zoom = 1.; }
                });
                egui::CollapsingHeader::new("Reordenar questões em todas as versões").show(ui, |ui| {
                    egui::ScrollArea::vertical().id_salt("pdf_question_order").max_height(160.).show(ui, |ui| {
                        let count = batch.students[*student].questions.len();
                        let mut change = None;
                        for (i, q) in batch.students[*student].questions.iter().enumerate() {
                            ui.push_id(i, |ui| { ui.horizontal(|ui| {
                                if position_button(ui, true, i > 0, "Adiantar questão").clicked() { change = Some((i, i-1)); }
                                if position_button(ui, false, i+1 < count, "Postergar questão").clicked() { change = Some((i, i+1)); }
                                ui.label(format!("{}. {}", i+1, q.title));
                            }); });
                        }
                        if let Some((a,b)) = change { moved = batch.move_question(a,b); self.invalidate(); }
                    });
                });
                if self.document.as_ref().is_none_or(|(i, _)| *i != *student) {
                    self.document = Some((*student, pdf::preview(batch, *student)));
                }
                let doc = &self.document.as_ref().unwrap().1;
                small(ui, format!("{} páginas · Mesma paginação, margens, imagens e conteúdo do PDF. A aparência das fontes pode variar ligeiramente.", doc.pages.len()));
                let scale = ((ui.available_width() - 32.) / pdf::PAGE_SIZE[0] as f32).min(1.35) * self.zoom;
                egui::ScrollArea::both().id_salt(("pdf_pages", *student)).auto_shrink([false, false])
                    .max_height((ui.available_height()-8.).max(120.)).show(ui, |ui| {
                        for page in &doc.pages { paint_page(ui, page, scale); ui.add_space(18.); }
                    });
            });
        self.open = open && !ctx.input(|i| i.key_pressed(egui::Key::Escape));
        moved
    }
}
fn paint_page(ui: &mut egui::Ui, page: &[pdf::PreviewOp], scale: f32) {
    let size = Vec2::new(pdf::PAGE_SIZE[0] as f32, pdf::PAGE_SIZE[1] as f32) * scale;
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    painter.rect_filled(rect, 0., C::WHITE);
    painter.rect_stroke(rect, 0., Stroke::new(1f32, LINE), egui::StrokeKind::Inside);
    let pos =
        |x: f64, y: f64| rect.min + Vec2::new(x as f32, (pdf::PAGE_SIZE[1] - y) as f32) * scale;
    let color = |v: [u8; 3]| C::from_rgb(v[0], v[1], v[2]);
    for op in page {
        match op {
            pdf::PreviewOp::Text {
                x,
                y,
                text,
                size,
                bold,
                color: ink,
            } => {
                let font = egui::FontId::new(
                    *size as f32 * scale,
                    egui::FontFamily::Name(if *bold { "pdf_bold" } else { "pdf_regular" }.into()),
                );
                let mut cursor = *x;
                for ch in text.chars() {
                    let glyph = ch.to_string();
                    // Use the PDF's character advances as well as its baselines and wrapping.
                    painter.text(
                        pos(cursor, *y + size * 0.93),
                        egui::Align2::LEFT_TOP,
                        &glyph,
                        font.clone(),
                        color(*ink),
                    );
                    cursor += pdf::preview_text_width(&glyph, *size, *bold);
                }
            }
            pdf::PreviewOp::Line {
                from,
                to,
                width,
                color: ink,
            } => {
                painter.line_segment(
                    [pos(from[0], from[1]), pos(to[0], to[1])],
                    Stroke::new(*width as f32 * scale, color(*ink)),
                );
            }
            pdf::PreviewOp::Rect {
                x,
                y,
                width,
                height,
                color: ink,
            } => {
                painter.rect_filled(
                    egui::Rect::from_min_size(
                        pos(*x, y + height),
                        Vec2::new(*width as f32, *height as f32) * scale,
                    ),
                    0.,
                    color(*ink),
                );
            }
            pdf::PreviewOp::Image {
                x,
                top,
                width,
                height,
                picture,
            } => {
                use std::hash::{Hash, Hasher};
                let mut hash = std::collections::hash_map::DefaultHasher::new();
                picture.data.hash(&mut hash);
                let id = egui::Id::new(("pdf_picture", hash.finish()));
                let texture = ui
                    .ctx()
                    .data_mut(|d| d.get_temp::<egui::TextureHandle>(id))
                    .or_else(|| {
                        let img = image::load_from_memory(&picture.data).ok()?;
                        let img = if img.width() > 1280 || img.height() > 1280 {
                            img.thumbnail(1280, 1280).to_rgba8()
                        } else {
                            img.to_rgba8()
                        };
                        let texture = ui.ctx().load_texture(
                            &picture.name,
                            egui::ColorImage::from_rgba_unmultiplied(
                                [img.width() as usize, img.height() as usize],
                                img.as_raw(),
                            ),
                            egui::TextureOptions::LINEAR,
                        );
                        ui.ctx().data_mut(|d| d.insert_temp(id, texture.clone()));
                        Some(texture)
                    });
                if let Some(texture) = texture {
                    painter.image(
                        texture.id(),
                        egui::Rect::from_min_size(
                            pos(*x, *top),
                            Vec2::new(*width as f32, *height as f32) * scale,
                        ),
                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1., 1.)),
                        C::WHITE,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn popup_draws_switches_student_and_closes_on_escape() {
        let ctx = egui::Context::default();
        ExamPreview::install_fonts(&ctx);
        let mut batch = generate(
            &examples(),
            &[("Mecânica".into(), 2)],
            "Ana\nBruno",
            42,
            "Prova",
            "Escola",
            "Maria",
            "12/11/2026",
            "A",
        )
        .unwrap();
        let mut popup = ExamPreview {
            open: true,
            ..Default::default()
        };
        let mut student = 0;
        for frame in 0..4 {
            if frame == 1 {
                student = 1;
            }
            if frame == 2 {
                assert!(batch.move_question(0, 1));
                popup.invalidate();
            }
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(1120., 740.),
                )),
                events: if frame == 3 {
                    vec![egui::Event::Key {
                        key: egui::Key::Escape,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: Default::default(),
                    }]
                } else {
                    vec![]
                },
                ..Default::default()
            };
            let result = ctx.run(input, |ctx| {
                popup.show(ctx, &mut batch, &mut student);
            });
            assert!(!result.shapes.is_empty());
            assert_eq!(popup.document.as_ref().unwrap().0, student);
        }
        assert!(!popup.open);
    }
}
