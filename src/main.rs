#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod model;
mod pdf;
mod updater;
use eframe::egui::{self, Color32 as C, Margin, RichText as R, Stroke, Vec2};
use model::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

const BG: C = C::from_rgb(245, 247, 249);
const INK: C = C::from_rgb(31, 45, 53);
const MUTED: C = C::from_rgb(112, 126, 136);
const TEAL: C = C::from_rgb(31, 112, 98);
const LINE: C = C::from_rgb(226, 232, 235);
const PALE: C = C::from_rgb(232, 244, 239);
const ERROR: C = C::from_rgb(190, 54, 54);
fn data_root() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let is_cargo = parent
                .parent()
                .and_then(|p| p.file_name())
                .map(|n| n == "target")
                .unwrap_or(false);
            if !is_cargo {
                if parent.join("installed.txt").exists() {
                    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
                        return PathBuf::from(local).join("Test Craft").join("data");
                    }
                }
                return parent.join("data");
            }
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
fn answer_label(index: usize) -> String {
    if index < 26 {
        ((b'a' + index as u8) as char).to_string()
    } else {
        (index + 1).to_string()
    }
}
fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(C::WHITE)
        .stroke(Stroke::new(1f32, LINE))
        .corner_radius(12)
        .inner_margin(20)
}
fn dot(ui: &mut egui::Ui, color: C) {
    let (r, _) = ui.allocate_exact_size(Vec2::new(8., 12.), egui::Sense::hover());
    ui.painter().circle_filled(r.center(), 3., color);
}
fn small(ui: &mut egui::Ui, s: impl Into<String>) {
    ui.label(R::new(s).size(12.).color(MUTED));
}
fn tag(ui: &mut egui::Ui, s: &str, color: C, bg: C) {
    egui::Frame::new()
        .fill(bg)
        .corner_radius(5)
        .inner_margin(Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.label(R::new(s).size(11.).color(color));
        });
}
fn primary(ui: &mut egui::Ui, s: &str) -> egui::Response {
    ui.add(
        egui::Button::new(R::new(s).color(C::WHITE).size(14.))
            .fill(TEAL)
            .stroke(Stroke::NONE)
            .corner_radius(7)
            .min_size(Vec2::new(130., 38.)),
    )
}
fn secondary(ui: &mut egui::Ui, s: &str) -> egui::Response {
    ui.add(
        egui::Button::new(R::new(s).size(13.).color(INK))
            .fill(C::WHITE)
            .stroke(Stroke::new(1f32, LINE))
            .corner_radius(7)
            .min_size(Vec2::new(92., 35.)),
    )
}
fn field(ui: &mut egui::Ui, label: &str, value: &mut String) {
    small(ui, label);
    ui.add(
        egui::TextEdit::singleline(value)
            .background_color(C::from_rgb(246, 248, 250))
            .desired_width(f32::INFINITY)
            .margin(Vec2::new(10., 9.)),
    );
    ui.add_space(9.);
}
fn show_pictures(ui: &mut egui::Ui, pictures: &[Picture]) {
    use std::hash::{Hash, Hasher};
    for p in pictures {
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        p.data.hash(&mut hash);
        let key = egui::Id::new(("picture", hash.finish()));
        let texture = ui
            .ctx()
            .data_mut(|d| d.get_temp::<egui::TextureHandle>(key));
        let texture = texture.or_else(|| {
            let img = image::load_from_memory(&p.data)
                .ok()?
                .thumbnail(1280, 1280)
                .to_rgba8();
            let size = [img.width() as usize, img.height() as usize];
            let t = ui.ctx().load_texture(
                &p.name,
                egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()),
                egui::TextureOptions::LINEAR,
            );
            ui.ctx().data_mut(|d| d.insert_temp(key, t.clone()));
            Some(t)
        });
        if let Some(t) = texture {
            ui.add(
                egui::Image::new(&t)
                    .max_width(ui.available_width())
                    .max_height(200.)
                    .maintain_aspect_ratio(true),
            );
        }
    }
}
fn pick_picture() -> Result<Option<Picture>, String> {
    if let Some(p) = rfd::FileDialog::new()
        .add_filter("Imagem", &["png", "jpg", "jpeg"])
        .pick_file()
    {
        let picture = Picture {
            name: p.file_name().unwrap_or_default().to_string_lossy().into(),
            data: std::fs::read(p).map_err(|e| e.to_string())?,
        };
        check_picture(&picture)?;
        Ok(Some(picture))
    } else {
        Ok(None)
    }
}
fn rule(ui: &mut egui::Ui) {
    ui.add_space(8.);
    ui.separator();
    ui.add_space(8.);
}
fn icon(ui: &mut egui::Ui, kind: usize, color: C, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
    let p = ui.painter();
    let r = rect.shrink(2.);
    let s = Stroke::new(1.6f32, color);
    match kind {
        0 => {
            p.rect_stroke(r, 3., s, egui::StrokeKind::Inside);
            p.line_segment(
                [
                    r.left_top() + Vec2::new(size * 0.29, 0.),
                    r.left_bottom() + Vec2::new(size * 0.29, 0.),
                ],
                s,
            );
            for y in [0.3, 0.55] {
                p.line_segment(
                    [
                        r.left_top() + Vec2::new(size * 0.48, size * y),
                        r.left_top() + Vec2::new(size * 0.72, size * y),
                    ],
                    s,
                );
            }
        }
        1 => {
            p.rect_stroke(r, 2., s, egui::StrokeKind::Inside);
            for y in [0.25, 0.48, 0.71] {
                p.line_segment(
                    [
                        r.left_top() + Vec2::new(4., size * y),
                        r.left_top() + Vec2::new(size * 0.65, size * y),
                    ],
                    s,
                );
            }
        }
        2 => {
            p.rect_stroke(
                egui::Rect::from_min_max(r.min + Vec2::new(0., 4.), r.max),
                3.,
                s,
                egui::StrokeKind::Inside,
            );
            p.line_segment([r.min + Vec2::new(0., 4.), r.min], s);
            p.line_segment([r.min, r.min + Vec2::new(size * 0.45, 0.)], s);
        }
        _ => {
            p.circle_stroke(r.center(), size * 0.35, s);
            p.line_segment([r.center(), r.center() + Vec2::new(0., -size * 0.2)], s);
            p.line_segment([r.center(), r.center() + Vec2::new(size * 0.2, 0.)], s);
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct Setup {
    title: String,
    institution: String,
    teacher: String,
    date: String,
    class_name: String,
    names: String,
    seed: u64,
    plan: Vec<(String, usize)>,
}
impl Default for Setup {
    fn default() -> Self {
        Self {
            title: "Avaliação de Física".into(),
            institution: String::new(),
            teacher: String::new(),
            date: String::new(),
            class_name: String::new(),
            names: String::new(),
            seed: 2026,
            plan: vec![],
        }
    }
}
struct App {
    bank: Bank,
    bank_errors: BTreeMap<u64, String>,
    data: PathBuf,
    page: usize,
    search: String,
    category: String,
    selected: Option<u64>,
    preview_seed: u64,
    draft: Question,
    edit_id: Option<u64>,
    validation: Option<Result<String, String>>,
    dirty: bool,
    setup: Setup,
    batch: Option<Batch>,
    student_preview: usize,
    history: Vec<PathBuf>,
    last_output: Option<PathBuf>,
    status: String,
    status_error: bool,
    delete: Option<u64>,
    discard_to: Option<usize>,
    help: bool,
    updater: updater::Updater,
    update_repository: String,
    update_automatic: bool,
    screenshot: Option<PathBuf>,
    frames: u32,
    screenshot_requested: bool,
}
impl App {
    fn new(cc: &eframe::CreationContext<'_>, screenshot: Option<PathBuf>, page: usize) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals = egui::Visuals::light();
        style.visuals.panel_fill = BG;
        style.visuals.text_edit_bg_color = Some(C::from_rgb(246, 248, 250));
        style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1f32, LINE);
        style.visuals.override_text_color = Some(INK);
        style.visuals.widgets.inactive.bg_fill = C::from_rgb(249, 250, 251);
        style.visuals.widgets.inactive.weak_bg_fill = C::from_rgb(249, 250, 251);
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1f32, LINE);
        style.visuals.widgets.hovered.bg_fill = PALE;
        style.visuals.widgets.active.bg_fill = PALE;
        style.visuals.selection.bg_fill = C::from_rgb(204, 228, 221);
        style.visuals.selection.stroke = Stroke::new(1f32, TEAL);
        style.spacing.item_spacing = Vec2::new(9., 9.);
        style.spacing.button_padding = Vec2::new(12., 7.);
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(14.));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(14.));
        style
            .text_styles
            .insert(egui::TextStyle::Heading, egui::FontId::proportional(25.));
        cc.egui_ctx.set_style(style);
        let data = data_root();
        let bankpath = data.join("banco.json");
        let (bank, error) = if bankpath.exists() {
            match read_bank(&bankpath) {
                Ok(b) => (b, None),
                Err(e) => (
                    Bank {
                        version: 1,
                        questions: vec![],
                    },
                    Some(format!(
                        "Não foi possível abrir o banco: {e} O arquivo foi preservado."
                    )),
                ),
            }
        } else {
            (examples(), None)
        };
        let setup = std::fs::read(data.join("preferencias.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        let updater = updater::Updater::load(&data);
        let update_repository = updater.settings.repository.clone();
        let update_automatic = updater.settings.automatic;
        let mut app = Self {
            bank_errors: bank
                .questions
                .iter()
                .filter_map(|q| validate(q).err().map(|e| (q.id, e)))
                .collect(),
            selected: bank.questions.first().map(|q| q.id),
            draft: examples().questions.remove(0),
            bank,
            updater,
            update_repository,
            update_automatic,
            data,
            page,
            search: String::new(),
            category: "Todas".into(),
            preview_seed: 17,
            edit_id: None,
            validation: None,
            dirty: false,
            setup,
            batch: None,
            student_preview: 0,
            history: vec![],
            last_output: None,
            status: error
                .clone()
                .unwrap_or_else(|| "Tudo pronto. Seus dados ficam neste computador.".into()),
            status_error: error.is_some(),
            delete: None,
            discard_to: None,
            help: false,
            screenshot,
            frames: 0,
            screenshot_requested: false,
        };
        app.sync_plan();
        app.refresh_history();
        if !bankpath.exists() && app.screenshot.is_none() {
            app.persist();
        }
        if page == 1 {
            app.begin_new();
        }
        app
    }
    fn notify(&mut self, msg: impl Into<String>, error: bool) {
        self.status = msg.into();
        self.status_error = error;
    }
    fn persist(&mut self) -> bool {
        self.batch = None;
        if let Err(e) = save_bank(&self.data.join("banco.json"), &self.bank) {
            self.notify(format!("Não foi possível salvar: {e}"), true);
            false
        } else {
            self.bank_errors = self
                .bank
                .questions
                .iter()
                .filter_map(|q| validate(q).err().map(|e| (q.id, e)))
                .collect();
            true
        }
    }
    fn sync_plan(&mut self) {
        let cats = categories(&self.bank);
        self.setup.plan.retain(|(c, _)| cats.contains(c));
        for c in cats {
            if !self.setup.plan.iter().any(|p| p.0 == c) {
                self.setup.plan.push((c, 0));
            }
        }
    }
    fn refresh_history(&mut self) {
        self.history = std::fs::read_dir(self.data.join("provas"))
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.join("lote.json").exists())
            .collect();
        self.history.sort();
        self.history.reverse();
    }
    fn navigate(&mut self, page: usize) {
        if self.page == 1 && self.dirty {
            self.discard_to = Some(page)
        } else {
            self.page = page
        }
    }
    fn begin_new(&mut self) {
        self.draft = examples().questions.remove(0);
        self.draft.id = now();
        self.draft.title.clear();
        self.edit_id = None;
        self.validation = None;
        self.dirty = false;
        self.page = 1;
    }
    fn edit(&mut self, q: Question) {
        self.edit_id = Some(q.id);
        self.draft = q;
        self.validation = None;
        self.dirty = false;
        self.page = 1;
    }
    fn import(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Banco de questões", &["json", "xml"])
            .pick_file()
        {
            let result = if path.extension().and_then(|s| s.to_str()) == Some("xml") {
                import_xml(&path)
            } else {
                read_bank(&path)
            };
            match result {
                Err(e) => self.notify(format!("Importação cancelada: {e}"), true),
                Ok(b) => {
                    let count = b.questions.len();
                    let errors = b.questions.iter().filter(|q| validate(q).is_err()).count();
                    let old = self.bank.clone();
                    let mut next = self.bank.questions.iter().map(|q| q.id).max().unwrap_or(0) + 1;
                    for mut q in b.questions {
                        q.id = next;
                        next += 1;
                        self.bank.questions.push(q)
                    }
                    if self.persist() {
                        self.sync_plan();
                        self.notify(
                            format!("{count} questões importadas e salvas. {errors} com erro, marcadas com borda vermelha."),
                            false,
                        )
                    } else {
                        self.bank = old
                    }
                }
            }
        }
    }
    fn export_bank(&mut self) {
        if let Some(p) = rfd::FileDialog::new()
            .add_filter("Banco JSON", &["json"])
            .set_file_name("banco_de_questoes.json")
            .save_file()
        {
            match save_bank(&p, &self.bank) {
                Ok(()) => self.notify(
                    "Banco exportado. O JSON pode ser aberto e importado em outro computador.",
                    false,
                ),
                Err(e) => self.notify(e, true),
            }
        }
    }
    fn sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("navigation")
            .exact_width(222.)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(C::from_rgb(23, 53, 49))
                    .inner_margin(Margin::symmetric(18, 26)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    egui::Frame::new()
                        .fill(C::from_rgb(75, 153, 131))
                        .corner_radius(9)
                        .inner_margin(8)
                        .show(ui, |ui| icon(ui, 0, C::WHITE, 24.));
                    ui.label(R::new("Test Craft").size(25.).strong().color(C::WHITE));
                });
                ui.add_space(6.);
                ui.label(
                    R::new("QUESTÕES QUE SE TRANSFORMAM")
                        .size(9.)
                        .color(C::from_rgb(148, 179, 170)),
                );
                ui.add_space(38.);
                ui.label(
                    R::new("SEU ESPAÇO")
                        .size(10.)
                        .color(C::from_rgb(140, 170, 162)),
                );
                ui.add_space(10.);
                for (page, kind, label) in [
                    (0, 0, "Banco de questões"),
                    (1, 1, "Elaborar questão"),
                    (2, 1, "Criar prova"),
                    (3, 2, "Arquivos locais"),
                    (4, 3, "Sobre e atualizações"),
                ] {
                    let selected = self.page == page;
                    let fill = if selected {
                        C::from_rgb(46, 82, 72)
                    } else {
                        C::TRANSPARENT
                    };
                    let frame = egui::Frame::new()
                        .fill(fill)
                        .corner_radius(7)
                        .inner_margin(Margin::symmetric(11, 12));
                    let r = frame
                        .show(ui, |ui| {
                            ui.set_width(164.);
                            ui.horizontal(|ui| {
                                icon(
                                    ui,
                                    kind,
                                    if selected {
                                        C::from_rgb(153, 222, 191)
                                    } else {
                                        C::from_rgb(157, 181, 173)
                                    },
                                    18.,
                                );
                                ui.label(R::new(label).size(13.).color(if selected {
                                    C::WHITE
                                } else {
                                    C::from_rgb(184, 203, 196)
                                }));
                            });
                        })
                        .response;
                    if ui
                        .interact(r.rect, r.id.with(page), egui::Sense::click())
                        .clicked()
                    {
                        if page == 1 && self.page != 1 {
                            self.begin_new()
                        } else {
                            self.navigate(page)
                        }
                    }
                    ui.add_space(3.);
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(
                        R::new("Feito para ensinar.")
                            .size(11.)
                            .color(C::from_rgb(130, 161, 151)),
                    );
                    ui.add_space(13.);
                    egui::Frame::new()
                        .fill(C::from_rgb(32, 65, 58))
                        .corner_radius(9)
                        .inner_margin(14)
                        .show(ui, |ui| {
                            ui.set_width(154.);
                            ui.horizontal(|ui| {
                                dot(ui, C::from_rgb(126, 211, 171));
                                ui.label(
                                    R::new("Seu banco é local")
                                        .size(12.)
                                        .color(C::from_rgb(216, 234, 225)),
                                );
                            });
                            ui.label(
                                R::new("Crie, salve e exporte.\nMesmo sem internet.")
                                    .size(11.)
                                    .color(C::from_rgb(151, 183, 170)),
                            );
                        });
                    if ui
                        .add(
                            egui::Button::new(
                                R::new("Como funciona   ?")
                                    .color(C::from_rgb(184, 203, 196))
                                    .size(12.),
                            )
                            .frame(false),
                        )
                        .clicked()
                    {
                        self.help = true;
                    }
                    ui.add_space(20.);
                });
            });
    }
    fn bank_page(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(R::new("Banco de questões").size(29.).strong());
                small(ui, "Uma boa questão. Inúmeras possibilidades.");
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if primary(ui, "+  Nova questão").clicked() {
                    self.begin_new()
                }
                if secondary(ui, "Importar banco").clicked() {
                    self.import()
                }
            });
        });
        ui.add_space(22.);
        let cats = categories(&self.bank);
        ui.columns(3, |cols| {
            for (i, (n, label, sub)) in [
                (
                    self.bank.questions.len().to_string(),
                    "Questões no banco",
                    "Prontas para suas próximas provas",
                ),
                (
                    cats.len().to_string(),
                    "Categorias",
                    "Organize por assunto ou disciplina",
                ),
                (
                    "100%".into(),
                    "Local e seu",
                    "Seus arquivos, no seu computador",
                ),
            ]
            .into_iter()
            .enumerate()
            {
                card().show(&mut cols[i], |ui| {
                    ui.horizontal(|ui| {
                        ui.label(R::new(n).size(28.).strong().color(if i == 2 {
                            TEAL
                        } else {
                            INK
                        }));
                        ui.add_space(5.);
                        ui.label(R::new(label).size(13.).strong());
                    });
                    small(ui, sub);
                });
            }
        });
        ui.add_space(22.);
        let w = ui.available_width();
        let right = 330f32.min(w * 0.39);
        let left = w - right - 20.;
        let content_height = ui.available_height();
        ui.horizontal_top(|ui|{
            ui.allocate_ui_with_layout(Vec2::new(left,content_height),egui::Layout::top_down(egui::Align::Min),|ui|{
                ui.horizontal(|ui|{
                    ui.add_sized([left-105.,37.],egui::TextEdit::singleline(&mut self.search).background_color(C::from_rgb(246,248,250)).hint_text("Buscar por título, assunto ou enunciado...").margin(Vec2::new(12.,10.)));
                    if secondary(ui,"Exportar").clicked(){self.export_bank()}
                });
                ui.add_space(8.);
                ui.horizontal_wrapped(|ui|{
                    for cat in std::iter::once("Todas".to_owned()).chain(cats.clone()){
                        let active=self.category==cat;
                        if ui.add(egui::Button::new(R::new(&cat).size(12.).color(if active{TEAL}else{MUTED})).fill(if active{PALE}else{C::TRANSPARENT}).stroke(Stroke::NONE).corner_radius(6)).clicked(){self.category=cat;}
                    }
                });
                ui.add_space(8.);
                let search=self.search.to_lowercase();
                let filtered:Vec<_>=self.bank.questions.iter().filter(|q|(self.category=="Todas"||self.category==q.category)&&format!("{} {} {}",q.title,q.category,q.text).to_lowercase().contains(&search)).cloned().collect();
                ui.horizontal(|ui|{small(ui,format!("{} questões encontradas",filtered.len()));});
                ui.add_space(4.);
                egui::ScrollArea::vertical().id_salt("banklist").max_height((content_height-142.).max(150.)).show(ui,|ui|{
                    if filtered.is_empty(){card().show(ui,|ui|{ui.label("Nenhuma questão encontrada.");small(ui,"Crie uma questão ou ajuste sua busca.");});}
                    for q in filtered{
                        let selected=self.selected==Some(q.id);
                        let error=self.bank_errors.get(&q.id).cloned();
                        let frame=egui::Frame::new().fill(if selected{C::from_rgb(246,252,249)}else{C::WHITE}).stroke(Stroke::new(if error.is_some(){2.}else if selected{1.5f32}else{1f32},if error.is_some(){ERROR}else if selected{C::from_rgb(93,159,137)}else{LINE})).corner_radius(9).inner_margin(16);
                        let response=frame.show(ui,|ui|{
                            ui.set_width((left-36.).max(100.));
                            ui.horizontal(|ui|{
                                egui::Frame::new().fill(if selected{PALE}else{BG}).corner_radius(7).inner_margin(8).show(ui,|ui|{ui.label(R::new("ƒx").size(18.).color(if selected{TEAL}else{MUTED}));});
                                ui.vertical(|ui|{
                                    ui.label(R::new(if q.title.trim().is_empty(){"Sem título"}else{&q.title}).size(15.).strong());
                                    if let Some(e)=&error{ui.colored_label(ERROR,"Com erro · Revisar").on_hover_text(e);}
                                    small(ui,format!("{}   ·   Calculada   ·   {} variáveis",q.category,q.variables.len()));
                                });
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{ui.label(R::new("›").size(23.).color(if selected{TEAL}else{MUTED}));});
                            });
                        }).response;
                        if ui.interact(response.rect,response.id.with(q.id),egui::Sense::click()).clicked(){self.selected=Some(q.id);self.preview_seed=17;}
                        ui.add_space(5.);
                    }
                });
            });
            ui.add_space(8.);
            ui.allocate_ui_with_layout(Vec2::new(right,content_height),egui::Layout::top_down(egui::Align::Min),|ui|{
                egui::ScrollArea::vertical().id_salt("bank_preview").show(ui,|ui|{
                if let Some(q)=self.bank.questions.iter().find(|q|Some(q.id)==self.selected).cloned(){
                    let error=self.bank_errors.get(&q.id).cloned();
                    card().stroke(Stroke::new(if error.is_some(){2f32}else{1f32},if error.is_some(){ERROR}else{LINE})).show(ui,|ui|{
                        ui.set_width(right-42.);
                        ui.horizontal(|ui|{small(ui,"PRÉVIA DA QUESTÃO");ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|tag(ui,"Exemplo",TEAL,PALE));});
                        ui.add_space(12.);ui.label(R::new(&q.title).size(20.).strong());
                        small(ui,&q.category);rule(ui);
                        if let Some(e)=&error{ui.colored_label(ERROR,"Com erro · Edite para corrigir");ui.label(e);ui.label(&q.text);}else if let Ok(p)=preview(&q,self.preview_seed){Self::show_instance(ui,&p,true);}
                        ui.add_space(12.);
                        if secondary(ui,"Sortear novos valores").clicked(){self.preview_seed+=1;}
                        rule(ui);
                        ui.horizontal(|ui|{if primary(ui,"Editar questão").clicked(){self.edit(q.clone())}if ui.button("···").on_hover_text("Duplicar ou excluir").clicked(){ui.memory_mut(|m|m.data.insert_temp(egui::Id::new("more"),true));}});
                        let more=ui.memory(|m|m.data.get_temp::<bool>(egui::Id::new("more")).unwrap_or(false));
                        if more{ui.horizontal(|ui|{
                            if ui.button("Duplicar").clicked(){let mut copy=q.clone();copy.id=now();copy.title.push_str(" (cópia)");self.edit_id=None;self.draft=copy;self.page=1;self.dirty=true;self.validation=None;}
                            if ui.button("Excluir").clicked(){self.delete=Some(q.id)}
                        });}
                    });
                    ui.add_space(12.);
                    egui::Frame::new().fill(C::from_rgb(236,242,248)).corner_radius(9).inner_margin(16).show(ui,|ui|{
                        ui.set_width(right-34.);ui.label(R::new("Uma questão, várias versões").size(12.).strong().color(C::from_rgb(73,101,125)));
                        ui.label(R::new("Na prova, os valores e a ordem das alternativas são sorteados para cada aluno.").size(12.).color(C::from_rgb(93,118,139)));
                    });
                }else{card().show(ui,|ui|{small(ui,"Selecione uma questão para ver a prévia.");});}
                });
            });
        });
    }
    fn show_instance(ui: &mut egui::Ui, p: &Instance, reveal: bool) {
        ui.label(R::new(&p.text).size(14.).line_height(Some(21.)));
        show_pictures(ui, &p.pictures);
        ui.add_space(12.);
        for (i, a) in p.answers.iter().enumerate() {
            let good = reveal && a.correct;
            egui::Frame::new()
                .fill(if good {
                    PALE
                } else {
                    C::from_rgb(248, 249, 250)
                })
                .corner_radius(6)
                .inner_margin(10)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            R::new(format!("{}.", answer_label(i)))
                                .strong()
                                .color(if good { TEAL } else { MUTED }),
                        );
                        ui.label(R::new(&a.text).color(if good { TEAL } else { INK }));
                        if good {
                            ui.label(R::new("correta").size(10.).color(TEAL));
                        }
                    });
                    show_pictures(ui, &a.pictures);
                });
        }
        if reveal && !p.values.is_empty() {
            ui.add_space(10.);
            small(ui, "VALORES DESTA PRÉVIA");
            ui.horizontal_wrapped(|ui| {
                for (k, v) in &p.values {
                    tag(ui, &format!("{k} = {}", fmt(*v)), MUTED, BG);
                }
            });
        }
    }
    fn editor_page(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    R::new(if self.edit_id.is_some() {
                        "Editar questão"
                    } else {
                        "Nova questão"
                    })
                    .size(29.)
                    .strong(),
                );
                small(ui, "Escreva uma vez. Deixe os números variarem.");
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if primary(ui, "Salvar questão").clicked() {
                    let result = validate(&self.draft);
                    let old = self.bank.clone();
                    let mut q = self.draft.clone();
                    q.title = q.title.trim().into();
                    q.category = q.category.trim().into();
                    if let Some(id) = self.edit_id {
                        if let Some(i) = self.bank.questions.iter().position(|q| q.id == id) {
                            self.bank.questions[i] = q.clone();
                        }
                    } else {
                        q.id = self.bank.questions.iter().map(|q| q.id).max().unwrap_or(0) + 1;
                        self.bank.questions.push(q.clone());
                    }
                    if self.persist() {
                        self.selected = Some(q.id);
                        self.sync_plan();
                        self.dirty = false;
                        self.page = 0;
                        self.notify(if result.is_ok() { "Questão validada e salva no banco local." } else { "Questão salva com erro. A borda vermelha indica que precisa de revisão." }, false);
                    } else {
                        self.bank = old;
                    }
                    self.validation = Some(result);
                }
                if secondary(ui, "Voltar ao banco").clicked() {
                    self.navigate(0)
                }
            });
        });
        ui.add_space(20.);
        let w = ui.available_width();
        let side = 310f32.min(w * 0.36);
        let content_height = ui.available_height();
        ui.horizontal_top(|ui|{
            ui.allocate_ui_with_layout(Vec2::new(w-side-22.,content_height),egui::Layout::top_down(egui::Align::Min),|ui|{
                egui::ScrollArea::vertical().id_salt("editor").max_height(ui.available_height()).show(ui,|ui|{
                    if !self.draft.import_issues.is_empty(){
                        card().stroke(Stroke::new(2f32,ERROR)).show(ui,|ui|{
                            ui.colored_label(ERROR,"Problemas na importação");
                            for issue in &self.draft.import_issues{ui.label(issue);}
                            if ui.button("Marcar problemas da importação como revisados").clicked(){self.draft.import_issues.clear();self.dirty=true;self.validation=Some(validate(&self.draft));}
                        });
                        ui.add_space(12.);
                    }
                    if let Some(xml)=&self.draft.source_xml{
                        if ui.small_button("Copiar XML original da questão").clicked(){ui.ctx().copy_text(xml.clone());}
                    }
                    card().show(ui,|ui|{
                        ui.label(R::new("1   Identificação e enunciado").size(16.).strong());ui.add_space(10.);
                        let before=self.draft.clone();
                        field(ui,"Título da questão",&mut self.draft.title);
                        field(ui,"Categoria / assunto",&mut self.draft.category);
                        small(ui,"Enunciado · use {d} para inserir uma variável");
                        ui.add(egui::TextEdit::multiline(&mut self.draft.text).background_color(C::from_rgb(246,248,250)).desired_width(f32::INFINITY).desired_rows(4).margin(Vec2::splat(10.)));
                        ui.horizontal(|ui|{
                            if ui.small_button("+ Imagem no enunciado").clicked(){match pick_picture(){Ok(Some(p))=>{self.draft.pictures.push(p);self.dirty=true;self.validation=None;},Ok(None)=>{},Err(e)=>self.validation=Some(Err(e))}}
                            small(ui,format!("{} imagens · PNG / JPG",self.draft.pictures.len()));
                        });
                        let mut remove=None;for (i,p) in self.draft.pictures.iter().enumerate(){ui.horizontal(|ui|{small(ui,&p.name);if ui.small_button("Remover").clicked(){remove=Some(i)}});}
                        if let Some(i)=remove{self.draft.pictures.remove(i);self.dirty=true;self.validation=None;}
                        if before.title!=self.draft.title||before.category!=self.draft.category||before.text!=self.draft.text{self.dirty=true;self.validation=None;}
                    });
                    ui.add_space(12.);
                    card().show(ui,|ui|{
                        ui.horizontal(|ui|{ui.label(R::new("2   Variáveis").size(16.).strong());ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{if ui.button("+ Adicionar").clicked(){self.draft.variables.push(Variable{name:format!("x{}",self.draft.variables.len()+1),min:1.,max:10.,decimals:0,values:vec![]});self.dirty=true;self.validation=None;}});});
                        small(ui,"Valores em passos de 1, 0,1 ou 0,01, conforme as casas decimais.");
                        ui.add_space(9.);let mut remove=None;
                        egui::Grid::new("vars").spacing([12.,10.]).num_columns(5).show(ui,|ui|{
                            for s in ["Nome","Mínimo","Máximo","Casas",""]{small(ui,s)}ui.end_row();
                            for (i,v) in self.draft.variables.iter_mut().enumerate(){
                                let mut changed=ui.add_sized([80.,30.],egui::TextEdit::singleline(&mut v.name).background_color(C::from_rgb(246,248,250))).changed();
                                changed|=ui.add(egui::DragValue::new(&mut v.min).speed(1.).range(-1e9..=1e9)).changed();
                                changed|=ui.add(egui::DragValue::new(&mut v.max).speed(1.).range(-1e9..=1e9)).changed();
                                changed|=ui.add(egui::DragValue::new(&mut v.decimals).range(0..=6)).changed();
                                if ui.button("×").clicked(){remove=Some(i)}
                                ui.end_row();
                                if !v.values.is_empty(){small(ui,format!("{} valores importados",v.values.len()));if ui.small_button("Usar intervalo").clicked(){v.values.clear();changed=true;}ui.end_row();}
                                if changed{self.dirty=true;self.validation=None;}
                            }
                        });
                        if let Some(i)=remove{self.draft.variables.remove(i);self.dirty=true;self.validation=None;}
                    });
                    ui.add_space(12.);
                    card().show(ui,|ui|{
                        ui.label(R::new("3   Alternativas e gabarito").size(16.).strong());
                        small(ui,"Use {={d}/{t}} para calcular. Marque a(s) alternativa(s) correta(s).");
                        ui.add_space(9.);let mut remove=None;
                        for (i,a) in self.draft.answers.iter_mut().enumerate(){
                            ui.horizontal(|ui|{
                                let mut changed=ui.checkbox(&mut a.correct,format!("{}.",answer_label(i))).on_hover_text("Alternativa correta").changed();
                                changed|=ui.add_sized([ui.available_width()-80.,32.],egui::TextEdit::singleline(&mut a.text).background_color(C::from_rgb(246,248,250)).font(egui::TextStyle::Monospace)).changed();
                                if ui.small_button("Img").on_hover_text("Adicionar imagem à alternativa").clicked(){match pick_picture(){Ok(Some(p))=>{a.pictures.push(p);changed=true;},Ok(None)=>{},Err(e)=>self.validation=Some(Err(e))}}
                                if ui.small_button("×").clicked(){remove=Some(i)}
                                if changed{self.dirty=true;self.validation=None;}
                            });
                        }
                        for (i,a) in self.draft.answers.iter_mut().enumerate(){
                            if !a.pictures.is_empty(){ui.horizontal(|ui|{small(ui,format!("Alternativa {}: {} imagem(ns)",answer_label(i),a.pictures.len()));if ui.small_button("Remover imagens").clicked(){a.pictures.clear();self.dirty=true;self.validation=None;}});}
                        }
                        if let Some(i)=remove{self.draft.answers.remove(i);self.dirty=true;self.validation=None;}
                        if self.draft.answers.len()<8&&ui.button("+ Adicionar alternativa").clicked(){self.draft.answers.push(Answer{text:String::new(),correct:false,pictures:vec![]});self.dirty=true;self.validation=None;}
                        rule(ui);if secondary(ui,"Validar questão").clicked(){self.validation=Some(validate(&self.draft))}
                    });
                });
            });
            ui.add_space(10.);
            ui.allocate_ui_with_layout(Vec2::new(side,content_height),egui::Layout::top_down(egui::Align::Min),|ui|{
                egui::ScrollArea::vertical().id_salt("editor_preview").show(ui,|ui|{
                    card().show(ui,|ui|{
                        small(ui,"PRÉVIA AO VIVO");ui.add_space(12.);
                        let safe= self.draft.variables.len()<=12 && self.draft.answers.len()<=8 && self.draft.variables.iter().all(|v|v.decimals<=6&&v.min.is_finite()&&v.max.is_finite()&&v.min<=v.max&&v.count()>0&&v.count()<usize::MAX/2);
                        if safe{match preview(&self.draft,self.preview_seed){Ok(p)=>Self::show_instance(ui,&p,true),Err(e)=>{ui.colored_label(C::from_rgb(173,89,50),e);}}}
                        else{small(ui,"Ajuste os intervalos para visualizar.");}
                        ui.add_space(10.);if secondary(ui,"Sortear valores").clicked(){self.preview_seed+=1;}
                    });
                    if let Some(result)=&self.validation{
                        ui.add_space(12.);egui::Frame::new().fill(if result.is_ok(){PALE}else{C::from_rgb(255,238,229)}).corner_radius(9).inner_margin(16).show(ui,|ui|{
                            ui.label(R::new(if result.is_ok(){"Validação concluída"}else{"Questão com erro · Pode salvar para revisar"}).strong().color(if result.is_ok(){TEAL}else{C::from_rgb(169,80,46)}));
                            ui.label(match result{Ok(s)=>s,Err(e)=>e});
                        });
                    }
                    ui.add_space(12.);
                    card().show(ui,|ui|{
                        ui.label(R::new("Ajuda com fórmulas").size(13.).strong());
                        small(ui,"Operações: +  -  *  /  ^\nFunções: sqrt, abs, sin, cos, tan,\nln, log, log10, exp\nConstantes: pi, e");
                        rule(ui);small(ui,"Exemplo de enunciado:\nUm carro percorre {d} m em {t} s.\n\nExemplo de alternativa:\n{={d}/{t}} m/s");
                        small(ui,"Use ponto ou vírgula decimal. Funções recebem um argumento. Os resultados aparecem com até duas casas.");
                    });
                });
            });
        });
    }
    fn build_batch(&mut self) -> bool {
        match generate(
            &self.bank,
            &self.setup.plan,
            &self.setup.names,
            self.setup.seed,
            &self.setup.title,
            &self.setup.institution,
            &self.setup.teacher,
            &self.setup.date,
            &self.setup.class_name,
        ) {
            Ok(b) => {
                self.batch = Some(b);
                self.student_preview = 0;
                self.notify(
                    "Prévia pronta. Confira uma versão por aluno antes de exportar.",
                    false,
                );
                true
            }
            Err(e) => {
                self.notify(e, true);
                false
            }
        }
    }
    fn proof_page(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui|{
            ui.vertical(|ui|{ui.label(R::new("Criar prova").size(29.).strong());small(ui,"Versões diferentes. A mesma facilidade para corrigir.");});
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
                if primary(ui,"Gerar PDFs").clicked(){
                    if self.batch.is_some()||self.build_batch(){
                        if let Some(base)=rfd::FileDialog::new().set_directory(self.data.join("provas")).pick_folder(){
                            match pdf::export(self.batch.as_ref().unwrap(),&base){Ok(p)=>{self.last_output=Some(p);self.notify("PDFs e gabaritos salvos com sucesso. Veja os arquivos locais.",false);self.refresh_history();},Err(e)=>self.notify(e,true)}
                        }
                    }
                }
                if secondary(ui,"Conferir prévia").clicked(){self.build_batch();}
            });
        });
        ui.add_space(20.);
        let w = ui.available_width();
        let side = (w * 0.43).min(440.);
        let content_height = ui.available_height();
        let mut changed = false;
        ui.horizontal_top(|ui|{
            ui.allocate_ui_with_layout(Vec2::new(w-side-22.,content_height),egui::Layout::top_down(egui::Align::Min),|ui|{
                egui::ScrollArea::vertical().id_salt("exam_setup").show(ui,|ui|{
                    card().show(ui,|ui|{
                        ui.label(R::new("1   Dados da prova").size(16.).strong());ui.add_space(10.);
                        let old=serde_json::to_string(&self.setup).unwrap_or_default();
                        field(ui,"Título / disciplina",&mut self.setup.title);field(ui,"Instituição",&mut self.setup.institution);
                        ui.columns(2,|cols|{field(&mut cols[0],"Professor(a)",&mut self.setup.teacher);field(&mut cols[1],"Turma / curso",&mut self.setup.class_name);});
                        field(ui,"Data",&mut self.setup.date);
                        changed|=old!=serde_json::to_string(&self.setup).unwrap_or_default();
                    });ui.add_space(12.);
                    card().show(ui,|ui|{
                        ui.label(R::new("2   Questões por categoria").size(16.).strong());small(ui,"Sorteio sem repetir questões dentro de cada prova.");ui.add_space(8.);
                        let len=self.setup.plan.len();let mut mv=None;
                        egui::Grid::new("category_plan").spacing([14.,10.]).show(ui,|ui|{
                            for s in ["Categoria","Disponíveis","Sortear","",""]{small(ui,s)}ui.end_row();
                            for (i,(cat,n)) in self.setup.plan.iter_mut().enumerate(){
                                let count=self.bank.questions.iter().filter(|q|&q.category==cat).count();
                                ui.label(cat.as_str());small(ui,count.to_string());changed|=ui.add(egui::DragValue::new(n).range(0..=count)).changed();
                                if ui.add_enabled(i>0,egui::Button::new("Subir").small()).clicked(){mv=Some((i,i-1))}
                                if ui.add_enabled(i+1<len,egui::Button::new("Descer").small()).clicked(){mv=Some((i,i+1))}
                                ui.end_row();
                            }
                        });
                        if let Some((a,b))=mv{self.setup.plan.swap(a,b);changed=true;}
                        if self.setup.plan.is_empty(){small(ui,"Adicione questões ao banco primeiro.");}
                        rule(ui);ui.label(R::new(format!("{} questões por aluno",self.setup.plan.iter().map(|p|p.1).sum::<usize>())).strong().color(TEAL));
                    });ui.add_space(12.);
                    card().show(ui,|ui|{
                        ui.label(R::new("3   Alunos").size(16.).strong());small(ui,"Cole a lista, um nome por linha. Use a matrícula para distinguir nomes iguais.");
                        changed|=ui.add(egui::TextEdit::multiline(&mut self.setup.names).background_color(C::from_rgb(246,248,250)).desired_width(f32::INFINITY).desired_rows(5).hint_text("Nome do aluno\nNome do aluno").margin(Vec2::splat(10.))).changed();
                        small(ui,format!("{} alunos na lista",self.setup.names.lines().filter(|s|!s.trim().is_empty()).count()));
                    });ui.add_space(12.);
                    card().show(ui,|ui|{
                        ui.horizontal(|ui|{small(ui,"Identificador do sorteio");changed|=ui.add(egui::DragValue::new(&mut self.setup.seed).range(0..=999_999_999u64)).changed();if ui.button("Novo").clicked(){self.setup.seed=now()%999_999_999;changed=true;}});
                        small(ui,"O mesmo banco, lista e identificador reproduzem as mesmas provas.");
                    });
                });
            });
            ui.add_space(10.);
            ui.allocate_ui_with_layout(Vec2::new(side,content_height),egui::Layout::top_down(egui::Align::Min),|ui|{
                egui::ScrollArea::vertical().id_salt("exam_preview").show(ui,|ui|{
                    card().show(ui,|ui|{
                        small(ui,"PRÉVIA INDIVIDUAL");
                        if let Some(batch)=&self.batch{
                            ui.add_space(8.);egui::ComboBox::from_id_salt("student").selected_text(&batch.students[self.student_preview].name).width(side-70.).show_ui(ui,|ui|{for (i,s) in batch.students.iter().enumerate(){ui.selectable_value(&mut self.student_preview,i,&s.name);}});
                            rule(ui);ui.label(R::new(&batch.title).size(19.).strong());small(ui,&batch.institution);small(ui,format!("{}   ·   {}",batch.class_name,batch.date));rule(ui);
                            for (i,q) in batch.students[self.student_preview].questions.iter().enumerate(){ui.label(R::new(format!("Questão {}",i+1)).strong());Self::show_instance(ui,q,false);ui.add_space(15.);}
                            rule(ui);small(ui,"GABARITO DESTA VERSÃO");
                            for (i,q) in batch.students[self.student_preview].questions.iter().enumerate(){ui.label(R::new(format!("Q{}   {}",i+1,pdf::letters(q))).color(TEAL));}
                        }else{
                            ui.add_space(25.);icon(ui,1,C::from_rgb(148,174,165),44.);ui.add_space(14.);ui.label(R::new("Sua prova começa aqui").size(18.).strong());
                            ui.label("Preencha os dados, escolha as quantidades e adicione os alunos.");
                            ui.add_space(10.);small(ui,"Clique em Conferir prévia para ver as versões individuais.");ui.add_space(30.);
                        }
                    });ui.add_space(12.);
                    egui::Frame::new().fill(PALE).corner_radius(9).inner_margin(18).show(ui,|ui|{
                        ui.label(R::new("O que será salvo").strong().color(TEAL));
                        small(ui,"• Uma prova em PDF por aluno\n• Um gabarito individual por aluno\n• Todas as provas em um PDF\n• Gabarito consolidado em PDF e CSV\n• Registro dos valores sorteados (JSON)");
                    });
                });
            });
        });
        if changed {
            self.batch = None;
            self.notify("Dados alterados. Confira uma nova prévia.", false);
        }
    }
    fn open_folder(&mut self, p: PathBuf) {
        match std::process::Command::new("explorer.exe").arg(&p).spawn() {
            Ok(_) => {}
            Err(e) => self.notify(format!("Não foi possível abrir a pasta: {e}"), true),
        }
    }
    fn files_page(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(R::new("Arquivos locais").size(29.).strong());
                small(ui, "Seu trabalho organizado, com você.");
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if primary(ui, "Exportar banco").clicked() {
                    self.export_bank()
                }
                if secondary(ui, "Importar").clicked() {
                    self.import()
                }
            });
        });
        ui.add_space(22.);
        egui::ScrollArea::vertical().show(ui,|ui|{
            card().show(ui,|ui|{
                ui.horizontal(|ui|{icon(ui,2,TEAL,28.);ui.vertical(|ui|{ui.label(R::new("Banco de questões").size(17.).strong());small(ui,format!("{} questões · formato JSON aberto e portátil",self.bank.questions.len()));});});
                rule(ui);small(ui,"SALVO AUTOMATICAMENTE NESTE COMPUTADOR");ui.label(self.data.join("banco.json").display().to_string());
                ui.add_space(8.);if secondary(ui,"Abrir pasta de dados").clicked(){self.open_folder(self.data.clone())}
                small(ui,"Use Exportar banco para guardar uma cópia ou levar suas questões para outro computador.");
                small(ui,"Importação: JSON deste aplicativo ou XML Moodle (calculatedmulti e multichoice), com imagens PNG/JPG incorporadas. Referências a imagens externas são recusadas com aviso.");
            });ui.add_space(18.);
            if let Some(p)=self.last_output.clone(){card().show(ui,|ui|{ui.label(R::new("Último lote exportado").size(17.).strong().color(TEAL));ui.label(p.display().to_string());if primary(ui,"Abrir arquivos").clicked(){self.open_folder(p.clone())}if ui.button("Carregar prévia deste lote").clicked(){self.load_batch(&p)}});ui.add_space(16.);}
            ui.label(R::new("Provas salvas na pasta padrão").size(19.).strong());small(ui,"Lotes exportados em outras pastas podem ser abertos pelo registro JSON.");
            ui.horizontal(|ui|{if secondary(ui,"Abrir registro de lote").clicked(){if let Some(p)=rfd::FileDialog::new().add_filter("Registro do lote",&["json"]).pick_file(){if let Some(parent)=p.parent(){self.load_batch_file(&p,parent.to_path_buf());}}}if ui.button("Atualizar").clicked(){self.refresh_history()}});
            for p in self.history.clone(){
                card().show(ui,|ui|{
                    let batch=std::fs::read(p.join("lote.json")).ok().and_then(|b|serde_json::from_slice::<Batch>(&b).ok());
                    if let Some(b)=batch{
                        ui.horizontal(|ui|{ui.vertical(|ui|{ui.label(R::new(&b.title).strong());small(ui,format!("{} alunos · identificador {}",b.students.len(),b.seed));});ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{if secondary(ui,"Abrir pasta").clicked(){self.open_folder(p.clone())}if ui.button("Ver versões").clicked(){self.load_batch(&p);}});});
                    }
                });ui.add_space(10.);
            }
            if self.history.is_empty(){ui.add_space(15.);small(ui,"Ainda não há lotes na pasta padrão. Crie sua primeira prova.");}
        });
    }
    fn load_batch(&mut self, p: &std::path::Path) {
        self.load_batch_file(&p.join("lote.json"), p.to_path_buf())
    }
    fn load_batch_file(&mut self, p: &std::path::Path, folder: PathBuf) {
        let result = std::fs::read(p)
            .map_err(|e| e.to_string())
            .and_then(|b| serde_json::from_slice::<Batch>(&b).map_err(|e| e.to_string()));
        match result {
            Ok(b)
                if b.version == 1
                    && !b.students.is_empty()
                    && b.students.iter().all(|s| {
                        !s.questions.is_empty()
                            && s.questions.iter().all(|q| {
                                (2..=8).contains(&q.answers.len())
                                    && q.answers.iter().any(|a| a.correct)
                            })
                    }) =>
            {
                if !b.plan.is_empty() {
                    self.setup.plan = b.plan.clone();
                } else {
                    let mut counts = std::collections::BTreeMap::new();
                    for q in &b.students[0].questions {
                        if let Some(original) =
                            self.bank.questions.iter().find(|x| x.title == q.title)
                        {
                            *counts.entry(original.category.clone()).or_insert(0usize) += 1;
                        }
                    }
                    self.setup.plan = counts.into_iter().collect();
                }
                self.sync_plan();
                self.setup.title = b.title.clone();
                self.setup.seed = b.seed;
                self.setup.names = b
                    .students
                    .iter()
                    .map(|s| s.name.clone())
                    .collect::<Vec<_>>()
                    .join("\n");
                self.setup.institution = b.institution.clone();
                self.setup.teacher = b.teacher.clone();
                self.setup.class_name = b.class_name.clone();
                self.setup.date = b.date.clone();
                self.batch = Some(b);
                self.student_preview = 0;
                self.last_output = Some(folder);
                self.page = 2;
                self.notify(
                    "Registro carregado com os valores e alternativas originais.",
                    false,
                );
            }
            Ok(_) => self.notify("Registro vazio ou incompatível.", true),
            Err(e) => self.notify(format!("Registro inválido: {e}"), true),
        }
    }
    fn updates_page(&mut self, ui: &mut egui::Ui) {
        ui.label(R::new("Sobre e atualizações").size(29.).strong());
        small(ui, "Novas versões, com seus arquivos sempre no computador.");
        ui.add_space(22.);
        egui::ScrollArea::vertical().show(ui, |ui| {
            card().show(ui, |ui| {
                ui.label(R::new(format!("Test Craft {}", updater::VERSION)).size(21.).strong());
                ui.label("Banco de questões calculadas e provas individualizadas.");
                small(ui, "Feito com o Art Craft.");
                small(ui, "Software livre · licença MIT · Windows 10/11, 64 bits");
                rule(ui);
                ui.label(R::new("Atualizações do programa").size(18.).strong());
                small(ui, "O aplicativo consulta releases públicas do GitHub ao abrir e a cada 6 horas enquanto estiver em uso.");
                let busy = self.updater.busy();
                ui.add_enabled_ui(!busy, |ui| {
                    field(ui, "Repositório ou link de releases", &mut self.updater.settings.repository);
                    ui.checkbox(&mut self.update_automatic, "Verificar automaticamente");
                    if primary(ui, "Salvar preferências").clicked() {
                        if let Err(e) = self.updater.save(&self.data, &self.update_repository, self.update_automatic) { self.updater.message = e; }
                    }
                });
                ui.add_space(8.);
                if ui.add_enabled(!busy, egui::Button::new("Verificar agora")).clicked() { self.updater.check(ui.ctx()); }
                if busy { ui.spinner(); }
                ui.label(&self.updater.message);
                if let Some(release) = self.updater.available.clone() {
                    if primary(ui, "Baixar atualização").clicked() { ui.ctx().open_url(egui::OpenUrl::new_tab(release.url)); }
                }
                if let Ok(repo) = updater::repository(&self.updater.settings.repository) {
                    ui.horizontal(|ui| {
                        ui.hyperlink_to("Código aberto", format!("https://github.com/{repo}"));
                        ui.hyperlink_to("Todas as releases", format!("https://github.com/{repo}/releases"));
                    });
                }
                rule(ui);
                small(ui, "Ao atualizar: feche o Test Craft e execute o novo MSI. Seus dados são preservados. Na versão portátil, substitua o executável mantendo a pasta data.");
                small(ui, "A consulta usa somente o nome do repositório e a versão do programa. Questões, alunos e provas ficam locais. Sem internet, continue trabalhando e tente a consulta depois.");
            });
            ui.add_space(18.);
            card().show(ui, |ui| {
                ui.label(R::new("Referências de produto").size(18.).strong());
                ui.label("Banco de questões, versões de prova, cabeçalhos e gabaritos orientam o fluxo simples do Test Craft.");
                ui.horizontal(|ui| {
                    ui.hyperlink_to("Gradepen", "https://gradepen.com/p/");
                    ui.hyperlink_to("Minha Prova", "https://www.minhaprova.com.br");
                });
                small(ui, "As referências não têm vínculo com o projeto. Recursos como correção óptica, TRI e provas online não integram esta versão.");
            });
        });
    }
    fn dialogs(&mut self, ctx: &egui::Context) {
        if self.updater.offer {
            if let Some(release) = self.updater.available.clone() {
                let mut open = true;
                egui::Window::new("Nova versão disponível")
                    .open(&mut open).collapsible(false).resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO).default_width(420.)
                    .show(ctx, |ui| {
                        ui.label(R::new(format!("Test Craft {} está disponível", release.version)).size(20.).strong().color(TEAL));
                        ui.label(format!("Versão instalada: {}", updater::VERSION));
                        ui.label("Baixe a nova versão na página oficial de releases e feche o aplicativo antes de executar o MSI. Seus dados são preservados.");
                        ui.horizontal(|ui| {
                            if primary(ui, "Baixar atualização").clicked() {
                                ctx.open_url(egui::OpenUrl::new_tab(release.url.clone()));
                                self.updater.offer = false;
                            }
                            if secondary(ui, "Mais tarde").clicked() { self.updater.offer = false; }
                        });
                    });
                if !open {
                    self.updater.offer = false;
                }
            }
        }
        if let Some(id) = self.delete {
            egui::Window::new("Excluir questão?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("A questão será removida do banco local.");
                    small(ui, "Provas já exportadas permanecem disponíveis.");
                    ui.horizontal(|ui| {
                        if secondary(ui, "Cancelar").clicked() {
                            self.delete = None;
                        }
                        if primary(ui, "Excluir questão").clicked() {
                            let old = self.bank.clone();
                            self.bank.questions.retain(|q| q.id != id);
                            if self.persist() {
                                self.selected = self.bank.questions.first().map(|q| q.id);
                                self.sync_plan();
                                self.notify("Questão excluída do banco.", false)
                            } else {
                                self.bank = old
                            }
                            self.delete = None;
                        }
                    });
                });
        }
        if let Some(page) = self.discard_to {
            egui::Window::new("Alterações não salvas")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("Deseja continuar editando ou descartar as alterações?");
                    ui.horizontal(|ui| {
                        if secondary(ui, "Continuar editando").clicked() {
                            self.discard_to = None;
                        }
                        if secondary(ui, "Descartar alterações").clicked() {
                            self.dirty = false;
                            self.page = page;
                            self.discard_to = None;
                        }
                    });
                });
        }
        if self.help {
            let mut open = true;
            egui::Window::new("Do banco à prova, em poucos passos").open(&mut open).resizable(false).default_width(440.).show(ctx,|ui|{
            for (title,text) in [("1. Elabore suas questões","Cadastre enunciado, variáveis e alternativas. Use {x} no texto e {={x}*2} para calcular. Questões com erro também podem ser salvas para revisão."),("2. Monte a prova","Escolha quantas questões sortear de cada categoria e cole os nomes dos alunos. Confira as versões na prévia."),("3. Salve e compartilhe","Exporte PDFs de provas e gabaritos. O banco pode ser exportado em JSON e importado em outro computador.")]{ui.label(R::new(title).strong().color(TEAL));ui.label(text);ui.add_space(14.);}
            small(ui,"Os exemplos iniciais são editáveis. Variáveis e alternativas são sorteadas por aluno; coincidências de valores entre alunos são possíveis, como em qualquer sorteio.");
        });
            self.help = open;
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frames += 1;
        if self.screenshot.is_none() {
            self.updater.tick(ctx);
        }
        if let Some(path) = &self.screenshot {
            for event in ctx.input(|i| i.events.clone()) {
                if let egui::Event::Screenshot { image, .. } = event {
                    if let Some(parent) = path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                    if let Err(e) = image::save_buffer(
                        path,
                        &bytes,
                        image.size[0] as u32,
                        image.size[1] as u32,
                        image::ColorType::Rgba8,
                    ) {
                        eprintln!("Screenshot: {e}");
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            if self.frames >= 120 && !self.screenshot_requested {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                self.screenshot_requested = true;
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }
        self.sidebar(ctx);
        egui::TopBottomPanel::top("topbar")
            .exact_height(60.)
            .frame(
                egui::Frame::new()
                    .fill(C::WHITE)
                    .inner_margin(Margin::symmetric(28, 18)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    small(ui, "Seu espaço");
                    ui.label(R::new("/").color(LINE));
                    ui.label(
                        R::new(
                            [
                                "Banco de questões",
                                "Elaborar questão",
                                "Criar prova",
                                "Arquivos locais",
                                "Sobre e atualizações",
                            ][self.page],
                        )
                        .size(12.),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        egui::Frame::new()
                            .fill(PALE)
                            .corner_radius(20)
                            .inner_margin(Margin::symmetric(10, 4))
                            .show(ui, |ui| {
                                ui.label(R::new("P").size(12.).strong().color(TEAL));
                            });
                        small(ui, "Professor(a)");
                        ui.add_space(12.);
                        dot(ui, TEAL);
                        small(ui, "Modo local");
                    });
                });
            });
        egui::TopBottomPanel::bottom("status")
            .min_height(35.)
            .frame(
                egui::Frame::new()
                    .fill(C::WHITE)
                    .stroke(Stroke::new(1f32, LINE))
                    .inner_margin(Margin::symmetric(28, 8)),
            )
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    dot(
                        ui,
                        if self.status_error {
                            C::from_rgb(173, 78, 48)
                        } else {
                            TEAL
                        },
                    );
                    ui.label(R::new(&self.status).size(11.).color(if self.status_error {
                        C::from_rgb(173, 78, 48)
                    } else {
                        MUTED
                    }));
                });
            });
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(BG)
                    .inner_margin(Margin::symmetric(28, 26)),
            )
            .show(ctx, |ui| match self.page {
                0 => self.bank_page(ui),
                1 => self.editor_page(ui),
                2 => self.proof_page(ui),
                3 => self.files_page(ui),
                _ => self.updates_page(ui),
            });
        self.dialogs(ctx);
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if self.screenshot.is_none() {
            let _ = std::fs::create_dir_all(&self.data);
            if let Ok(bytes) = serde_json::to_vec_pretty(&self.setup) {
                let _ = std::fs::write(self.data.join("preferencias.json"), bytes);
            }
        }
    }
}
fn main() -> eframe::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.iter().any(|a| a == "--self-check") {
        let bank = examples();
        for q in &bank.questions {
            validate(q).expect("example must validate");
        }
        let batch = generate(
            &bank,
            &[("Mecânica".into(), 2), ("Eletricidade".into(), 1)],
            "Ana Souza\nBruno Lima\nJoão Santos",
            2026,
            "Avaliação de Física",
            "Escola Exemplo",
            "Professor(a)",
            "07/11/2026",
            "Turma A",
        )
        .unwrap();
        let folder = pdf::export(
            &batch,
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("checks"),
        )
        .unwrap();
        println!("SELF-CHECK OK: {}", folder.display());
        return Ok(());
    }
    let screenshot = args
        .iter()
        .position(|a| a == "--screenshot")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);
    let page = args
        .iter()
        .position(|a| a == "--page")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0)
        .min(4);
    let open_batch = args
        .iter()
        .position(|a| a == "--open-batch")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1380., 900.])
            .with_min_inner_size([1120., 740.])
            .with_title("Test Craft · Provas individualizadas"),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Test Craft",
        options,
        Box::new(move |cc| {
            let mut app = App::new(cc, screenshot, page);
            if let Some(p) = open_batch {
                let folder = p
                    .parent()
                    .unwrap_or(std::path::Path::new("."))
                    .to_path_buf();
                app.load_batch_file(&p, folder);
            }
            Ok(Box::new(app))
        }),
    )
}
