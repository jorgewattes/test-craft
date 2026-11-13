#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod bank_view;
mod category_view;
mod exam_preview;
mod export_history;
mod import_view;
mod model;
mod moodle;
mod pdf;
mod storage;
mod student_grid;
mod updater;
use eframe::egui::{self, Color32 as C, Margin, RichText as R, Stroke, Vec2};
use model::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

const BG: C = C::from_rgb(245, 247, 249);
const INK: C = C::from_rgb(31, 45, 53);
const MUTED: C = C::from_rgb(112, 126, 136);
const TEAL: C = C::from_rgb(31, 112, 98);
const LINE: C = C::from_rgb(226, 232, 235);
const PALE: C = C::from_rgb(232, 244, 239);
const INPUT_BG: C = C::from_rgb(246, 248, 250);
const CONTROL_HEIGHT: f32 = 38.;
const PAGE_GAP: f32 = 20.;
const ERROR: C = C::from_rgb(190, 54, 54);
const DRAFT_BG: C = C::from_rgb(255, 249, 219);
const DRAFT_LINE: C = C::from_rgb(238, 217, 137);
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
fn page_title(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    ui.vertical(|ui| {
        ui.label(R::new(title).size(29.).strong());
        small(ui, subtitle);
    });
}
fn primary(ui: &mut egui::Ui, s: &str) -> egui::Response {
    ui.add(
        egui::Button::new(R::new(s).color(C::WHITE).size(14.))
            .fill(TEAL)
            .stroke(Stroke::NONE)
            .corner_radius(8)
            .min_size(Vec2::new(130., CONTROL_HEIGHT)),
    )
}
fn secondary(ui: &mut egui::Ui, s: &str) -> egui::Response {
    ui.add(
        egui::Button::new(R::new(s).size(14.).color(INK))
            .fill(C::WHITE)
            .stroke(Stroke::new(1f32, LINE))
            .corner_radius(8)
            .min_size(Vec2::new(92., CONTROL_HEIGHT)),
    )
}
fn draft_selectable_value<Value: PartialEq>(
    ui: &mut egui::Ui,
    current: &mut Value,
    value: Value,
    label: &str,
) -> egui::Response {
    ui.scope(|ui| {
        let visuals = ui.visuals_mut();
        visuals.selection.bg_fill = DRAFT_BG;
        visuals.selection.stroke = Stroke::new(1f32, C::from_rgb(139, 105, 20));
        visuals.widgets.hovered.weak_bg_fill = DRAFT_BG;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1f32, DRAFT_LINE);
        visuals.widgets.active.weak_bg_fill = DRAFT_BG;
        visuals.widgets.active.bg_stroke = Stroke::new(1f32, DRAFT_LINE);
        ui.selectable_value(current, value, label)
    }).inner
}
fn error_selectable_value<Value: PartialEq>(
    ui: &mut egui::Ui,
    current: &mut Value,
    value: Value,
    label: &str,
) -> egui::Response {
    ui.scope(|ui| {
        let visuals = ui.visuals_mut();
        let fill = C::from_rgb(255, 232, 232);
        visuals.selection.bg_fill = fill;
        visuals.selection.stroke = Stroke::new(1f32, ERROR);
        visuals.widgets.hovered.weak_bg_fill = fill;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1f32, ERROR);
        visuals.widgets.active.weak_bg_fill = fill;
        visuals.widgets.active.bg_stroke = Stroke::new(1f32, ERROR);
        ui.selectable_value(current, value, label)
    }).inner
}
fn difficulty_tag(ui: &mut egui::Ui, difficulty: Option<QuestionDifficulty>) {
    let (label, color, bg) = match difficulty {
        Some(QuestionDifficulty::Example) => ("Exemplo", C::from_rgb(77, 104, 148), C::from_rgb(235, 242, 252)),
        Some(QuestionDifficulty::Easy) => ("Fácil", TEAL, PALE),
        Some(QuestionDifficulty::Medium) => ("Média", C::from_rgb(139, 105, 20), DRAFT_BG),
        Some(QuestionDifficulty::Hard) => ("Difícil", C::from_rgb(167, 84, 28), C::from_rgb(255, 239, 225)),
        Some(QuestionDifficulty::Challenge) => ("Desafio", C::from_rgb(121, 69, 157), C::from_rgb(244, 233, 252)),
        None => ("Sem classificação", MUTED, BG),
    };
    tag(ui, label, color, bg);
}
fn field(ui: &mut egui::Ui, label: &str, value: &mut String) {
    small(ui, label);
    ui.add(
        egui::TextEdit::singleline(value)
            .background_color(INPUT_BG)
            .desired_width(f32::INFINITY)
            .margin(Vec2::new(10., 9.)),
    );
    ui.add_space(9.);
}
fn category_options(
    ui: &mut egui::Ui,
    selected: &mut String,
    choices: &[String],
    counts: &BTreeMap<String, usize>,
) {
    for path in choices {
        let depth = path.matches('/').count();
        let leaf = path.rsplit('/').next().unwrap_or(path);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.;
            ui.add_space((depth.min(8) * 16) as f32);
            if depth > 0 {
                let (r, _) = ui.allocate_exact_size(Vec2::new(10., 18.), egui::Sense::hover());
                ui.painter()
                    .line_segment([r.left_top(), r.left_center()], Stroke::new(1f32, LINE));
                ui.painter()
                    .line_segment([r.left_center(), r.right_center()], Stroke::new(1f32, LINE));
            }
            ui.selectable_value(
                selected,
                path.clone(),
                R::new(format!(
                    "{leaf} ({})",
                    counts.get(path).copied().unwrap_or(0)
                ))
                .strong(),
            )
            .on_hover_text(path.replace('/', " / "));
        });
    }
}
fn category_dropdown(
    ui: &mut egui::Ui,
    id: &str,
    selected: &mut String,
    choices: &[String],
    all: bool,
    counts: &BTreeMap<String, usize>,
) {
    let total: usize = counts
        .iter()
        .filter(|(path, _)| !path.contains('/'))
        .map(|(_, count)| count)
        .sum();
    egui::ComboBox::from_id_salt(id)
        .icon(|ui, rect, _visuals, _is_open| {
            ui.painter().line_segment(
                [
                    rect.center() + Vec2::new(-4., -2.),
                    rect.center() + Vec2::new(0., 2.),
                ],
                Stroke::new(1.5f32, TEAL),
            );
            ui.painter().line_segment(
                [
                    rect.center() + Vec2::new(0., 2.),
                    rect.center() + Vec2::new(4., -2.),
                ],
                Stroke::new(1.5f32, TEAL),
            );
        })
        .selected_text(if selected.is_empty() {
            "Selecione uma categoria".into()
        } else {
            if selected == "Todas" {
                format!("Todas as categorias ({total})")
            } else {
                format!(
                    "{} ({})",
                    selected.replace('/', " / "),
                    counts.get(selected).copied().unwrap_or(0)
                )
            }
        })
        .width(ui.available_width())
        .height(300.)
        .show_ui(ui, |ui| {
            if all {
                ui.selectable_value(
                    selected,
                    "Todas".into(),
                    format!("Todas as categorias ({total})"),
                );
            }
            category_options(ui, selected, choices, counts);
        });
}
fn position_button(ui: &mut egui::Ui, up: bool, enabled: bool, label: &str) -> egui::Response {
    let response = ui.add_enabled(enabled, egui::Button::new("").min_size(Vec2::splat(28.)));
    let center = response.rect.center();
    let direction = if up { -1. } else { 1. };
    let tip = center + Vec2::new(0., direction * 6.);
    let stroke = Stroke::new(
        1.7f32,
        if enabled {
            TEAL
        } else {
            C::from_rgb(180, 190, 195)
        },
    );
    let painter = ui.painter();
    painter.line_segment([center - Vec2::new(0., direction * 6.), tip], stroke);
    painter.line_segment([tip, tip + Vec2::new(-4., -direction * 4.)], stroke);
    painter.line_segment([tip, tip + Vec2::new(4., -direction * 4.)], stroke);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
    response.on_hover_text(label)
}
fn category_settings(ui: &mut egui::Ui) -> egui::Response {
    let (r, response) = ui.allocate_exact_size(Vec2::splat(38.), egui::Sense::click());
    ui.painter()
        .rect_filled(r, 7., if response.hovered() { PALE } else { C::WHITE });
    ui.painter()
        .rect_stroke(r, 7., Stroke::new(1f32, LINE), egui::StrokeKind::Inside);
    for (i, offset) in [-7., 0., 7.].iter().enumerate() {
        let y = r.center().y + offset;
        ui.painter().line_segment(
            [egui::pos2(r.left() + 9., y), egui::pos2(r.right() - 9., y)],
            Stroke::new(1.5f32, TEAL),
        );
        ui.painter().circle_filled(
            egui::pos2(r.left() + if i == 1 { 22. } else { 13. }, y),
            2.8,
            TEAL,
        );
    }
    let response = response.on_hover_text("Configurar categorias");
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Configurar categorias")
    });
    response
}
struct ImportReview {
    bank: Bank,
    errors: BTreeMap<u64, String>,
    source: String,
    destinations: BTreeMap<String, String>,
    overrides: BTreeMap<u64, String>,
}
fn display_preview(q: &Question, seed: u64) -> Result<Instance, String> {
    if q.variables.len() > 12
        || q.answers.len() > 8
        || q.variables.iter().any(|v| {
            v.decimals > 6
                || !v.min.is_finite()
                || !v.max.is_finite()
                || v.min > v.max
                || v.count() == 0
                || v.count() >= usize::MAX / 2
        })
    {
        return Err("Ajuste os intervalos para visualizar.".into());
    }
    review_preview(q, seed)
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
        4 => {
            p.rect_stroke(r, 2., s, egui::StrokeKind::Inside);
            p.line_segment(
                [
                    r.center() - Vec2::new(5., 0.),
                    r.center() + Vec2::new(5., 0.),
                ],
                s,
            );
            p.line_segment(
                [
                    r.center() - Vec2::new(0., 5.),
                    r.center() + Vec2::new(0., 5.),
                ],
                s,
            );
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
    #[serde(default)]
    logo: Option<Picture>,
    names: String,
    #[serde(default)]
    student_grid: Option<student_grid::StudentGrid>,
    seed: u64,
    plan: Vec<(String, usize)>,
    #[serde(default)]
    same_questions: bool,
    #[serde(default)]
    question_order: Vec<usize>,
}
impl Default for Setup {
    fn default() -> Self {
        Self {
            title: "Avaliação de Física".into(),
            institution: String::new(),
            teacher: String::new(),
            date: String::new(),
            class_name: String::new(),
            logo: None,
            names: String::new(),
            student_grid: None,
            seed: 2026,
            plan: vec![],
            same_questions: false,
            question_order: vec![],
        }
    }
}
struct App {
    context: egui::Context,
    store: Option<storage::BankStore>,
    index: bank_view::BankIndex,
    list_page: usize,
    validator: bank_view::Validator,
    validation_generation: u64,
    pending_validation: BTreeSet<u64>,
    bank: Bank,
    bank_errors: BTreeMap<u64, String>,
    bank_examples: BTreeMap<u64, ValidationExample>,
    data: PathBuf,
    page: usize,
    search: String,
    category: String,
    question_filter: bank_view::QuestionFilter,
    difficulty_filter: bank_view::DifficultyFilter,
    selected: Option<u64>,
    preview_seed: u64,
    draft: Question,
    edit_id: Option<u64>,
    validation: Option<Result<String, String>>,
    validation_example: Option<ValidationExample>,
    dirty: bool,
    setup: Setup,
    batch: Option<Batch>,
    student_preview: usize,
    exam_preview: exam_preview::ExamPreview,
    history: Vec<export_history::HistoryEntry>,
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
    screenshot_started: std::time::Instant,
    sidebar_collapsed: bool,
    category_dialog: bool,
    category_checked: BTreeSet<String>,
    category_bulk_destination: String,
    category_bulk_delete: bool,
    category_search: String,
    proof_category_search: String,
    proof_category_dialog: bool,
    proof_category_focus: bool,
    export_dialog: Option<Bank>,
    category_delete: Option<String>,
    category_delete_destination: String,
    import_review: Option<ImportReview>,
    import_loading: Option<std::sync::mpsc::Receiver<Result<ImportReview, String>>>,
    import_errors: bool,
    category_for_editor: bool,
    category_name: String,
    category_parent: String,
    category_error: Option<String>,
    category_edit: Option<String>,
    checked: BTreeSet<u64>,
    transfer: Option<bool>,
    transfer_destination: String,
    batch_delete: Option<BTreeSet<u64>>,
    expanded_preview: bool,
}
impl App {
    fn new(cc: &eframe::CreationContext<'_>, screenshot: Option<PathBuf>, page: usize) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals = egui::Visuals::light();
        style.visuals.panel_fill = BG;
        style.visuals.text_edit_bg_color = Some(INPUT_BG);
        style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1f32, LINE);
        style.visuals.override_text_color = Some(INK);
        style.visuals.widgets.inactive.bg_fill = C::from_rgb(249, 250, 251);
        style.visuals.widgets.inactive.weak_bg_fill = C::from_rgb(249, 250, 251);
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1f32, LINE);
        style.visuals.widgets.hovered.bg_fill = PALE;
        style.visuals.widgets.hovered.weak_bg_fill = PALE;
        style.visuals.widgets.active.weak_bg_fill = PALE;
        style.visuals.widgets.open.weak_bg_fill = PALE;
        style.visuals.widgets.active.bg_fill = PALE;
        style.visuals.selection.bg_fill = C::from_rgb(204, 228, 221);
        style.visuals.selection.stroke = Stroke::new(1f32, TEAL);
        style.spacing.item_spacing = Vec2::new(9., 9.);
        style.spacing.button_padding = Vec2::new(12., 8.);
        style.spacing.interact_size.y = CONTROL_HEIGHT;
        style.spacing.window_margin = Margin::same(20);
        style.spacing.combo_height = 300.;
        for widget in [
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
            &mut style.visuals.widgets.open,
        ] {
            widget.corner_radius = egui::CornerRadius::same(8);
            widget.bg_stroke = Stroke::new(1f32, LINE);
        }
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1f32, TEAL);
        style.visuals.widgets.active.bg_stroke = Stroke::new(1f32, TEAL);
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
        exam_preview::ExamPreview::install_fonts(&cc.egui_ctx);
        let data = data_root();
        let (store, mut bank, error) = match storage::BankStore::open(&data) {
            Ok((store, bank)) => (Some(store), bank, None),
            Err(e) => (
                None,
                Bank {
                    version: 1,
                    questions: vec![],
                    category_labels: vec![],
                },
                Some(format!(
                    "Não foi possível abrir o banco: {e} Os arquivos foram preservados."
                )),
            ),
        };
        bank.category_labels = categories(&bank);
        let setup = std::fs::read(data.join("preferencias.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        let updater = updater::Updater::load(&data);
        let update_repository = updater.settings.repository.clone();
        let update_automatic = updater.settings.automatic;
        let validator = bank_view::Validator::new(cc.egui_ctx.clone());
        let validation_generation = validator.submit(&bank);
        let mut app = Self {
            context: cc.egui_ctx.clone(),
            store,
            index: bank_view::BankIndex::new(&bank),
            list_page: 0,
            validator,
            validation_generation,
            pending_validation: bank.questions.iter().map(|q| q.id).collect(),
            bank_errors: BTreeMap::new(),
            bank_examples: BTreeMap::new(),
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
            question_filter: bank_view::QuestionFilter::All,
            difficulty_filter: bank_view::DifficultyFilter::All,
            preview_seed: 17,
            edit_id: None,
            validation: None,
            validation_example: None,
            dirty: false,
            setup,
            batch: None,
            student_preview: 0,
            exam_preview: Default::default(),
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
            screenshot_started: std::time::Instant::now(),
            sidebar_collapsed: std::fs::read(data_root().join("menu-recolhido.json"))
                .ok()
                .and_then(|b| serde_json::from_slice(&b).ok())
                .unwrap_or(false),
            category_dialog: false,
            category_checked: BTreeSet::new(),
            category_bulk_destination: String::new(),
            category_bulk_delete: false,
            category_search: String::new(),
            proof_category_search: String::new(),
            proof_category_dialog: false,
            proof_category_focus: false,
            export_dialog: None,
            category_delete: None,
            category_delete_destination: String::new(),
            import_review: None,
            import_loading: None,
            import_errors: true,
            category_for_editor: false,
            category_name: String::new(),
            category_parent: String::new(),
            category_error: None,
            category_edit: None,
            checked: BTreeSet::new(),
            transfer: None,
            transfer_destination: String::new(),
            batch_delete: None,
            expanded_preview: false,
        };
        if let Ok(message) = std::fs::read_to_string(app.data.join("update-result.txt")) {
            app.status_error = message.starts_with("Falha");
            app.status = message;
            let _ = std::fs::remove_file(app.data.join("update-result.txt"));
        }
        app.sync_plan();
        app.refresh_history();
        if page == 1 {
            app.begin_new();
        }
        app
    }
    fn notify(&mut self, msg: impl Into<String>, error: bool) {
        self.status = msg.into();
        self.status_error = error;
    }
    fn persist(&mut self, previous: &Bank) -> bool {
        self.bank.category_labels = categories(&self.bank);
        let result = self
            .store
            .as_mut()
            .ok_or_else(|| "O banco não está disponível. Reabra o aplicativo.".to_owned())
            .and_then(|store| store.save(&self.bank, Some(previous)));
        if let Err(e) = result {
            self.notify(format!("Não foi possível salvar: {e}"), true);
            false
        } else {
            self.batch = None;
            self.index = bank_view::BankIndex::new(&self.bank);
            self.bank_errors
                .retain(|id, _| self.index.positions.contains_key(id));
            self.bank_examples.clear();
            self.pending_validation = self.bank.questions.iter().map(|q| q.id).collect();
            self.validation_generation = self.validator.submit(&self.bank);
            true
        }
    }
    fn sync_plan(&mut self) {
        let cats = self.index.categories.clone();
        let previous = self.setup.plan.clone();
        self.setup.plan.retain(|(c, _)| cats.iter().any(|cat| cat == proof_pool_parts(c).0));
        self.setup.plan.retain(|(_, n)| *n > 0);
        if self.setup.plan != previous {
            self.setup.question_order.clear();
        }
    }
    fn refresh_history(&mut self) {
        self.history = export_history::list(&self.data);
    }
    fn remember_batch(&self, folder: &std::path::Path) -> Result<(), String> {
        export_history::save(
            &self.data,
            folder,
            self.batch.as_ref().ok_or("Não há prévia gerada.")?,
        )
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
        self.draft.status = QuestionStatus::Draft;
        self.draft.category = self
            .index
            .categories
            .clone()
            .into_iter()
            .next()
            .unwrap_or_default();
        self.edit_id = None;
        self.validation = None;
        self.validation_example = None;
        self.dirty = false;
        self.page = 1;
    }
    fn edit(&mut self, q: Question) {
        self.edit_id = Some(q.id);
        self.draft = q;
        self.validation = self.bank_errors.get(&self.draft.id).cloned().map(Err);
        self.validation_example = self.bank_examples.get(&self.draft.id).cloned();
        self.dirty = false;
        self.page = 1;
    }
    fn import(&mut self) {
        if let Some(paths) = rfd::FileDialog::new()
            .add_filter("Banco de questões", &["json", "xml"])
            .pick_files()
        {
            let (tx, rx) = std::sync::mpsc::channel();
            let ctx = self.context.clone();
            let remaining = 10000usize.saturating_sub(self.bank.questions.len());
            std::thread::spawn(move || {
                let result = import_view::read_import(paths, remaining);
                let _ = tx.send(result);
                ctx.request_repaint();
            });
            self.import_loading = Some(rx);
            self.import_errors = true;
        }
    }
    fn export_bank(&mut self) {
        self.export_dialog = Some(self.bank.clone());
    }
    fn sidebar(&mut self, ctx: &egui::Context) {
        let compact = self.sidebar_collapsed;
        egui::SidePanel::left("navigation")
            .exact_width(if compact { 72. } else { 222. })
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(C::from_rgb(23, 53, 49))
                    .inner_margin(Margin::symmetric(if compact { 12 } else { 18 }, 22)),
            )
            .show(ctx, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(Vec2::new(44., 38.), egui::Sense::click());
                for offset in [-6., 0., 6.] {
                    ui.painter().line_segment(
                        [
                            rect.center() + Vec2::new(-9., offset),
                            rect.center() + Vec2::new(9., offset),
                        ],
                        Stroke::new(1.8f32, C::WHITE),
                    );
                }
                if response
                    .on_hover_text(if compact {
                        "Expandir menu"
                    } else {
                        "Recolher menu"
                    })
                    .clicked()
                {
                    self.sidebar_collapsed = !compact;
                }
                if !compact {
                    ui.add_space(12.);
                    ui.horizontal(|ui| {
                        let t = ui
                            .ctx()
                            .data_mut(|d| {
                                d.get_temp::<egui::TextureHandle>(egui::Id::new("app_icon"))
                            })
                            .unwrap_or_else(|| {
                                let img = image::load_from_memory(include_bytes!(
                                    "../assets/test-craft.png"
                                ))
                                .unwrap()
                                .to_rgba8();
                                let t = ui.ctx().load_texture(
                                    "Test Craft",
                                    egui::ColorImage::from_rgba_unmultiplied(
                                        [img.width() as usize, img.height() as usize],
                                        img.as_raw(),
                                    ),
                                    egui::TextureOptions::LINEAR,
                                );
                                ui.ctx().data_mut(|d| {
                                    d.insert_temp(egui::Id::new("app_icon"), t.clone())
                                });
                                t
                            });
                        ui.add(egui::Image::new(&t).fit_to_exact_size(Vec2::splat(30.)));
                        ui.label(R::new("Test Craft").size(25.).strong().color(C::WHITE));
                    });
                    ui.label(
                        R::new("QUESTÕES QUE SE TRANSFORMAM")
                            .size(9.)
                            .color(C::from_rgb(148, 179, 170)),
                    );
                }
                ui.add_space(26.);
                for (page, kind, label) in [
                    (0, 0, "Banco de questões"),
                    (1, 1, "Elaborar questão"),
                    (2, 4, "Criar prova"),
                    (3, 2, "Arquivos locais"),
                    (4, 3, "Sobre e atualizações"),
                ] {
                    let selected = self.page == page;
                    let r = egui::Frame::new()
                        .fill(if selected {
                            C::from_rgb(46, 82, 72)
                        } else {
                            C::TRANSPARENT
                        })
                        .corner_radius(8)
                        .inner_margin(Margin::symmetric(11, 12))
                        .show(ui, |ui| {
                            ui.set_width(if compact { 22. } else { 164. });
                            ui.horizontal(|ui| {
                                icon(
                                    ui,
                                    kind,
                                    if selected {
                                        C::from_rgb(153, 222, 191)
                                    } else {
                                        C::from_rgb(157, 181, 173)
                                    },
                                    20.,
                                );
                                if !compact {
                                    ui.label(R::new(label).size(13.).color(if selected {
                                        C::WHITE
                                    } else {
                                        C::from_rgb(184, 203, 196)
                                    }));
                                }
                            });
                        })
                        .response;
                    if ui
                        .interact(r.rect, r.id.with(page), egui::Sense::click())
                        .on_hover_text(label)
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
                    if !compact {
                        ui.label(
                            R::new("Seu banco é local")
                                .size(12.)
                                .color(C::from_rgb(184, 203, 196)),
                        );
                        ui.label(
                            R::new("Crie, salve e exporte.\nMesmo sem internet.")
                                .size(11.)
                                .color(C::from_rgb(151, 183, 170)),
                        );
                    }
                    if ui
                        .add(
                            egui::Button::new(
                                R::new(if compact { "?" } else { "Como funciona   ?" })
                                    .color(C::from_rgb(184, 203, 196)),
                            )
                            .frame(false),
                        )
                        .on_hover_text("Como funciona")
                        .clicked()
                    {
                        self.help = true;
                    }
                });
            });
    }
    fn open_category_dialog(&mut self, editor: bool) {
        self.category_edit = None;
        self.category_for_editor = editor;
        self.category_parent.clear();
        self.category_name.clear();
        self.category_error = None;
        self.category_dialog = true;
    }
    fn edit_category_path(&mut self, path: String) {
        let (parent, name) = path.rsplit_once('/').unwrap_or(("", &path));
        self.category_name = name.into();
        self.category_parent = parent.into();
        self.category_edit = Some(path);
        self.category_error = None;
    }
    fn selection_toolbar(&mut self, ui: &mut egui::Ui, filtered: &[usize]) {
        self.checked
            .retain(|id| self.index.positions.contains_key(id));
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().interact_size.y = 28.;
            ui.spacing_mut().button_padding = Vec2::new(6., 4.);
            if ui
                .button("Selecionar resultados")
                .on_hover_text("Seleciona todas as questões da busca, incluindo as outras páginas")
                .clicked()
            {
                self.checked
                    .extend(filtered.iter().map(|&i| self.bank.questions[i].id));
            }
            if ui.button("Limpar seleção").clicked() {
                self.checked.clear();
            }
            small(ui, format!("{} selecionadas no banco", self.checked.len()));
            if !self.checked.is_empty() {
                ui.menu_button("Ações", |ui| {
                    if ui.button("Exportar seleção").clicked() {
                        self.export_dialog = Some(selected_bank(&self.bank, &self.checked));
                        ui.close();
                    }
                    for (label, copy) in [("Copiar", true), ("Mover", false)] {
                        if ui.button(label).clicked() {
                            self.transfer_destination.clear();
                            self.transfer = Some(copy);
                            ui.close();
                        }
                    }
                    if ui.button("Apagar").clicked() {
                        self.batch_delete = Some(self.checked.clone());
                        ui.close();
                    }
                    if ui.button("Copiar JSON").clicked() {
                        match serde_json::to_string_pretty(&selected_bank(
                            &self.bank,
                            &self.checked,
                        )) {
                            Ok(json) => {
                                ui.ctx().copy_text(json);
                                self.notify("Seleção copiada para a área de transferência.", false);
                            }
                            Err(e) => self.notify(e.to_string(), true),
                        }
                        ui.close();
                    }
                });
            }
        });
    }
    fn bank_page(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            page_title(ui, "Banco de questões", "Uma boa questão. Inúmeras possibilidades.");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if primary(ui, "+  Nova questão").clicked() {
                    self.begin_new()
                }
                if secondary(ui, "Importar banco").clicked() {
                    self.import()
                }
            });
        });
        ui.add_space(8.);
        let cats = self.index.categories.clone();
        small(
            ui,
            format!(
                "{} questões  ·  {} categorias  ·  Salvo neste computador",
                self.bank.questions.len(),
                cats.len()
            ),
        );
        ui.add_space(12.);
        let w = ui.available_width();
        let right = (w * 0.48).max(380.).min(w - 360.);
        let left = w - right - 20.;
        let content_height = ui.available_height();
        ui.horizontal_top(|ui|{
            ui.allocate_ui_with_layout(Vec2::new(left,content_height),egui::Layout::top_down(egui::Align::Min),|ui|{
                ui.horizontal(|ui|{
                    ui.add_sized([left-105.,37.],egui::TextEdit::singleline(&mut self.search).background_color(INPUT_BG).hint_text("Buscar por título, assunto ou enunciado...").margin(Vec2::new(12.,10.)));
                    if secondary(ui,"Exportar").clicked(){self.export_bank()}
                });
                ui.add_space(8.);
                small(ui,"Categoria / assunto");
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(Vec2::new((ui.available_width()-47.).max(100.),38.),egui::Layout::top_down(egui::Align::Min),|ui| {
                        category_dropdown(ui,"bank_category",&mut self.category,&cats,true,&self.index.category_counts);
                    });
                    if category_settings(ui).clicked(){self.open_category_dialog(false);}
                });
                ui.add_space(8.);
                ui.horizontal_wrapped(|ui| {
                    small(ui, "Situação");
                    for status in [bank_view::QuestionFilter::All, bank_view::QuestionFilter::Ready, bank_view::QuestionFilter::Error, bank_view::QuestionFilter::Draft] {
                        if status == bank_view::QuestionFilter::Draft {
                            draft_selectable_value(ui, &mut self.question_filter, status, status.label());
                        } else if status == bank_view::QuestionFilter::Error {
                            error_selectable_value(ui, &mut self.question_filter, status, status.label());
                        } else { ui.selectable_value(&mut self.question_filter, status, status.label()); }
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    small(ui, "Classificação");
                    egui::ComboBox::from_id_salt("bank_difficulty")
                        .selected_text(self.difficulty_filter.label())
                        .show_ui(ui, |ui| {
                            use bank_view::DifficultyFilter;
                            ui.selectable_value(&mut self.difficulty_filter, DifficultyFilter::All, DifficultyFilter::All.label());
                            ui.selectable_value(&mut self.difficulty_filter, DifficultyFilter::Unclassified, DifficultyFilter::Unclassified.label());
                            for level in QuestionDifficulty::ALL {
                                ui.selectable_value(&mut self.difficulty_filter, DifficultyFilter::Level(level), level.label());
                            }
                        });
                });
                let (filtered, filter_changed) = self.index.filter(&self.bank, &self.category, &self.search, self.question_filter, self.difficulty_filter, &self.bank_errors, &self.pending_validation);
                if filter_changed { self.list_page = 0; }
                ui.horizontal(|ui|{small(ui,format!("{} questões encontradas",filtered.len()));});
                self.selection_toolbar(ui, &filtered);
                let pages = filtered.len().div_ceil(bank_view::PAGE_SIZE).max(1);
                self.list_page = self.list_page.min(pages - 1);
                if pages > 1 {
                    ui.horizontal(|ui| {
                        if ui.add_enabled(self.list_page > 0, egui::Button::new("Anterior")).clicked() { self.list_page -= 1; }
                        small(ui, format!("Página {} de {}", self.list_page+1, pages));
                        if ui.add_enabled(self.list_page+1 < pages, egui::Button::new("Próxima")).clicked() { self.list_page += 1; }
                    });
                }
                if !self.pending_validation.is_empty() { small(ui, format!("Validando {} questões em segundo plano…",self.pending_validation.len())); }
                ui.add_space(4.);
                egui::ScrollArea::vertical().id_salt(("banklist",self.list_page)).max_height(ui.available_height()).show(ui,|ui|{
                    if filtered.is_empty(){card().show(ui,|ui|{ui.label("Nenhuma questão encontrada.");small(ui,"Crie uma questão ou ajuste sua busca.");});}
                    for &position in filtered.iter().skip(self.list_page * bank_view::PAGE_SIZE).take(bank_view::PAGE_SIZE) {
                        let q = &self.bank.questions[position];
                        let selected=self.selected==Some(q.id);
                        let error=self.bank_errors.get(&q.id).cloned();
                        let frame=egui::Frame::new().fill(if selected{C::from_rgb(246,252,249)}else{C::WHITE}).stroke(Stroke::new(if error.is_some(){2.}else if selected{1.5f32}else{1f32},if q.status==QuestionStatus::Draft{DRAFT_LINE}else if error.is_some(){ERROR}else if selected{C::from_rgb(93,159,137)}else{LINE})).corner_radius(9).inner_margin(10);
                        let mut checkbox_rect = egui::Rect::NOTHING;
                        let response=frame.show(ui,|ui|{
                            ui.set_width((left-24.).max(100.));
                            ui.horizontal(|ui|{
                                let mut checked = self.checked.contains(&q.id);
                                let check = ui.checkbox(&mut checked, "").on_hover_text("Selecionar para ações em lote");
                                checkbox_rect = check.rect;
                                if check.changed() { if checked {self.checked.insert(q.id);} else {self.checked.remove(&q.id);} }
                                ui.label(R::new("ƒx").size(14.).color(if selected{TEAL}else{MUTED}));
                                ui.vertical(|ui|{
                                    ui.label(R::new(if q.title.trim().is_empty(){"Sem título"}else{&q.title}).size(15.).strong());
                                    let status = if error.is_some() {"Com erro · Revisar"} else if q.status==QuestionStatus::Draft {"Rascunho"} else if self.pending_validation.contains(&q.id) {"Validando…"} else {"Pronta"};
                                    let summary=format!("{} · {status}",q.category);
                                    ui.label(R::new(summary).size(12.).color(if error.is_some(){ERROR}else{MUTED})).on_hover_text(error.as_deref().unwrap_or("Abra para conferir a questão"));
                                    difficulty_tag(ui, q.difficulty);
                                });
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{ui.label(R::new("›").size(23.).color(if selected{TEAL}else{MUTED}));});
                            });
                        }).response;
                        let preview_rect = egui::Rect::from_min_max(egui::pos2(checkbox_rect.right()+4., response.rect.top()), response.rect.max);
                        if ui.interact(preview_rect,response.id.with(q.id),egui::Sense::click()).clicked(){self.selected=Some(q.id);self.preview_seed=17;}
                        ui.add_space(5.);
                    }
                });
            });
            ui.add_space(8.);
            ui.allocate_ui_with_layout(Vec2::new(right,content_height),egui::Layout::top_down(egui::Align::Min),|ui|{
                ui.set_width(right);
                egui::ScrollArea::vertical().id_salt("bank_preview").auto_shrink([false,false]).show(ui,|ui|{
                if let Some(q)=self.selected.and_then(|id|self.index.positions.get(&id)).map(|&i|self.bank.questions[i].clone()){
                    let error=self.bank_errors.get(&q.id).cloned();
                    card().stroke(Stroke::new(if error.is_some(){2f32}else{1f32},if error.is_some(){ERROR}else{LINE})).show(ui,|ui|{
                        ui.set_width(right-42.);
                        ui.horizontal_wrapped(|ui|{small(ui,"PRÉVIA DA QUESTÃO");difficulty_tag(ui, q.difficulty);});
                        if ui.button("Ampliar prévia").clicked(){self.expanded_preview=true;}
                        ui.add_space(12.);ui.label(R::new(&q.title).size(20.).strong());
                        small(ui,&q.category);rule(ui);
                        if let Some(e)=&error{ui.colored_label(ERROR,"Com erro · Edite para corrigir");ui.label(e);Self::show_question_preview(ui,&q,self.preview_seed,self.bank_examples.get(&q.id));}else if self.pending_validation.contains(&q.id){small(ui,"Validando questão…");ui.label(&q.text);}else{Self::show_question_preview(ui,&q,self.preview_seed,None);}
                        ui.add_space(12.);
                        if !self.bank_examples.contains_key(&q.id) && secondary(ui,"Sortear novos valores").clicked(){self.preview_seed+=1;}
                        rule(ui);
                        ui.horizontal(|ui|{if primary(ui,"Editar questão").clicked(){self.edit(q.clone())}if ui.button("···").on_hover_text("Duplicar ou excluir").clicked(){ui.memory_mut(|m|m.data.insert_temp(egui::Id::new("more"),true));}});
                        let more=ui.memory(|m|m.data.get_temp::<bool>(egui::Id::new("more")).unwrap_or(false));
                        if more{ui.horizontal(|ui|{
                            if ui.button("Duplicar").clicked(){let mut copy=q.clone();copy.id=now();copy.title.push_str(" (cópia)");self.edit_id=None;self.draft=copy;self.page=1;self.dirty=true;self.validation=None; self.validation_example=None;}
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
        if !p.category.is_empty() {
            small(ui, p.category.replace('/', " / "));
        }
        ui.label(R::new(&p.text).size(14.).line_height(Some(21.)));
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.set_width(ui.available_width());
            show_pictures(ui, &p.pictures);
        });
        ui.add_space(12.);
        let duplicates = if reveal {
            duplicate_pairs(p)
        } else {
            Vec::new()
        };
        if !duplicates.is_empty() {
            ui.colored_label(
                ERROR,
                "Alternativas iguais neste exemplo · ajuste as fórmulas ou os intervalos.",
            );
        }
        for (i, a) in p.answers.iter().enumerate() {
            let duplicate = duplicates.iter().any(|&(a, b)| a == i || b == i);
            let good = reveal && a.correct;
            egui::Frame::new()
                .stroke(Stroke::new(if duplicate { 2f32 } else { 0f32 }, ERROR))
                .fill(if good {
                    PALE
                } else {
                    C::from_rgb(248, 249, 250)
                })
                .corner_radius(6)
                .inner_margin(10)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
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
                    if duplicate {
                        ui.colored_label(ERROR, "Resultado repetido");
                    }
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
    fn validate_draft(&mut self) -> Result<String, String> {
        let (result, example) = validate_with_example(&self.draft);
        self.validation_example = example;
        result
    }
    fn show_question_preview(
        ui: &mut egui::Ui,
        q: &Question,
        seed: u64,
        example: Option<&ValidationExample>,
    ) {
        let result = if let Some(example) = example {
            small(ui, "EXEMPLO QUE CAUSOU O ERRO · valores da validação");
            example_preview(q, example)
        } else {
            display_preview(q, seed)
        };
        match result {
            Ok(p) => Self::show_instance(ui, &p, true),
            Err(e) => {
                ui.colored_label(ERROR, e);
                ui.label(&q.text);
            }
        }
    }
    fn editor_page(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            page_title(ui, if self.edit_id.is_some() { "Editar questão" } else { "Nova questão" }, "Escreva uma vez. Deixe os números variarem.");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if primary(ui, "Salvar questão").clicked() {
                    let result = self.validate_draft();
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
                    if self.persist(&old) {
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
        ui.add_space(PAGE_GAP);
        let w = ui.available_width();
        let side = (w * 0.44).max(380.).min(w - 440.);
        let content_height = ui.available_height();
        ui.horizontal_top(|ui|{
            ui.allocate_ui_with_layout(Vec2::new(w-side-22.,content_height),egui::Layout::top_down(egui::Align::Min),|ui|{
                ui.set_width(w-side-22.);
                egui::ScrollArea::both().id_salt("editor").auto_shrink([false,false]).max_height(ui.available_height()).show(ui,|ui|{
                    if !self.draft.import_issues.is_empty(){
                        card().stroke(Stroke::new(2f32,ERROR)).show(ui,|ui|{
                            ui.colored_label(ERROR,"Problemas na importação");
                            for issue in &self.draft.import_issues{ui.label(issue);}
                            if ui.button("Marcar problemas da importação como revisados").clicked(){self.draft.import_issues.clear();self.dirty=true;self.validation=Some(self.validate_draft());}
                        });
                        ui.add_space(12.);
                    }
                    if let Some(xml)=&self.draft.source_xml{
                        if ui.button("Copiar XML original da questão").clicked(){ui.ctx().copy_text(xml.clone());}
                    }
                    card().stroke(Stroke::new(1f32,if self.draft.status==QuestionStatus::Draft{DRAFT_LINE}else{LINE})).show(ui,|ui|{
                        ui.label(R::new("1   Identificação e enunciado").size(16.).strong());ui.add_space(10.);
                        let before=self.draft.clone();
                        field(ui,"Título da questão",&mut self.draft.title);
                        ui.horizontal(|ui| {
                            small(ui,"Status da questão");
                            draft_selectable_value(ui,&mut self.draft.status,QuestionStatus::Draft,"Rascunho");
                            ui.selectable_value(&mut self.draft.status,QuestionStatus::Ready,"Pronta");
                        });
                        small(ui,"Somente questões prontas e sem erros participam das provas.");
                        ui.horizontal_wrapped(|ui| {
                            small(ui,"Classificação");
                            egui::ComboBox::from_id_salt("editor_difficulty")
                                .selected_text(self.draft.difficulty.map_or("Sem classificação", QuestionDifficulty::label))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.draft.difficulty, None, "Sem classificação");
                                    for level in QuestionDifficulty::ALL {
                                        ui.selectable_value(&mut self.draft.difficulty, Some(level), level.label());
                                    }
                                });
                        });
                        ui.add_space(9.);
                        small(ui,"Categoria / assunto");
                        ui.horizontal(|ui| {
                            ui.allocate_ui_with_layout(Vec2::new((ui.available_width()-47.).max(100.),38.),egui::Layout::top_down(egui::Align::Min),|ui| {
                                category_dropdown(ui,"editor_category",&mut self.draft.category,&self.index.categories,false,&self.index.category_counts);
                            });
                            if category_settings(ui).clicked(){self.open_category_dialog(true);}
                        });
                        ui.add_space(9.);
                        small(ui,"Enunciado · use {d} para inserir uma variável");
                        ui.add(egui::TextEdit::multiline(&mut self.draft.text).background_color(INPUT_BG).desired_width(f32::INFINITY).desired_rows(4).margin(Vec2::splat(10.)));
                        ui.horizontal(|ui|{
                            if ui.button("+ Imagem no enunciado").clicked(){match pick_picture(){Ok(Some(p))=>{self.draft.pictures.push(p);self.dirty=true;self.validation=None; self.validation_example=None;},Ok(None)=>{},Err(e)=>self.validation=Some(Err(e))}}
                            small(ui,format!("{} imagens · PNG / JPG",self.draft.pictures.len()));
                        });
                        let mut remove=None;for (i,p) in self.draft.pictures.iter().enumerate(){ui.horizontal(|ui|{small(ui,&p.name);if ui.button("Remover").clicked(){remove=Some(i)}});}
                        if let Some(i)=remove{self.draft.pictures.remove(i);self.dirty=true;self.validation=None; self.validation_example=None;}
                        if before.status!=self.draft.status||before.difficulty!=self.draft.difficulty||before.title!=self.draft.title||before.category!=self.draft.category||before.text!=self.draft.text{self.dirty=true;self.validation=None; self.validation_example=None;}
                    });
                    ui.add_space(12.);
                    card().show(ui,|ui|{
                        ui.horizontal(|ui|{ui.label(R::new("2   Variáveis").size(16.).strong());ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{if ui.button("+ Adicionar").clicked(){self.draft.variables.push(Variable{name:format!("x{}",self.draft.variables.len()+1),min:1.,max:10.,decimals:0,values:vec![]});self.dirty=true;self.validation=None; self.validation_example=None;}});});
                        small(ui,"Valores em passos de 1, 0,1 ou 0,01, conforme as casas decimais.");
                        ui.add_space(9.);let mut remove=None;
                        egui::Grid::new("vars").spacing([12.,10.]).num_columns(5).show(ui,|ui|{
                            for s in ["Nome","Mínimo","Máximo","Casas",""]{small(ui,s)}ui.end_row();
                            for (i,v) in self.draft.variables.iter_mut().enumerate(){
                                let mut changed=ui.add_sized([80.,30.],egui::TextEdit::singleline(&mut v.name).background_color(INPUT_BG)).changed();
                                changed|=ui.add(egui::DragValue::new(&mut v.min).speed(1.).range(-1e9..=1e9)).changed();
                                changed|=ui.add(egui::DragValue::new(&mut v.max).speed(1.).range(-1e9..=1e9)).changed();
                                changed|=ui.add(egui::DragValue::new(&mut v.decimals).range(0..=6)).changed();
                                if ui.button("×").clicked(){remove=Some(i)}
                                ui.end_row();
                                if !v.values.is_empty(){small(ui,format!("{} valores importados",v.values.len()));if ui.button("Usar intervalo").clicked(){v.values.clear();changed=true;}ui.end_row();}
                                if changed{self.dirty=true;self.validation=None; self.validation_example=None;}
                            }
                        });
                        if let Some(i)=remove{self.draft.variables.remove(i);self.dirty=true;self.validation=None; self.validation_example=None;}
                    });
                    ui.add_space(12.);
                    card().show(ui,|ui|{
                        ui.label(R::new("3   Alternativas e gabarito").size(16.).strong());
                        small(ui,"Use {={d}/{t}} para calcular. Marque a(s) alternativa(s) correta(s).");
                        ui.add_space(9.);let mut remove=None;
                        for (i,a) in self.draft.answers.iter_mut().enumerate(){
                            ui.horizontal(|ui|{
                                let mut changed=ui.checkbox(&mut a.correct,format!("{}.",answer_label(i))).on_hover_text("Alternativa correta").changed();
                                changed|=ui.add_sized([(ui.available_width()-128.).max(80.),CONTROL_HEIGHT],egui::TextEdit::singleline(&mut a.text).background_color(INPUT_BG).font(egui::TextStyle::Monospace)).changed();
                                if ui.button("Img").on_hover_text("Adicionar imagem à alternativa").clicked(){match pick_picture(){Ok(Some(p))=>{a.pictures.push(p);changed=true;},Ok(None)=>{},Err(e)=>self.validation=Some(Err(e))}}
                                if ui.button("×").clicked(){remove=Some(i)}
                                if changed{self.dirty=true;self.validation=None; self.validation_example=None;}
                            });
                        }
                        for (i,a) in self.draft.answers.iter_mut().enumerate(){
                            if !a.pictures.is_empty(){ui.horizontal(|ui|{small(ui,format!("Alternativa {}: {} imagem(ns)",answer_label(i),a.pictures.len()));if ui.button("Remover imagens").clicked(){a.pictures.clear();self.dirty=true;self.validation=None; self.validation_example=None;}});}
                        }
                        if let Some(i)=remove{self.draft.answers.remove(i);self.dirty=true;self.validation=None; self.validation_example=None;}
                        if self.draft.answers.len()<8&&ui.button("+ Adicionar alternativa").clicked(){self.draft.answers.push(Answer{text:String::new(),correct:false,pictures:vec![]});self.dirty=true;self.validation=None; self.validation_example=None;}
                        rule(ui);if secondary(ui,"Validar questão").clicked(){self.validation=Some(self.validate_draft())}
                    });
                });
            });
            ui.add_space(10.);
            ui.allocate_ui_with_layout(Vec2::new(side,content_height),egui::Layout::top_down(egui::Align::Min),|ui|{
                ui.set_width(side);
                egui::ScrollArea::both().id_salt("editor_preview").auto_shrink([false,false]).show(ui,|ui|{
                    card().show(ui,|ui|{
                        ui.set_width(side-42.);
                        small(ui,"PRÉVIA AO VIVO");
                        if ui.button("Ampliar prévia").clicked(){self.expanded_preview=true;}
                        ui.add_space(12.);
                        Self::show_question_preview(ui,&self.draft,self.preview_seed,self.validation_example.as_ref());
                        ui.add_space(10.);if self.validation_example.is_none() && secondary(ui,"Sortear valores").clicked(){self.preview_seed+=1;}
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
                        small(ui,"Operações: +  -  *  /  ^\nFunções: sqrt, abs, sin, cos, tan,\nln, log, log10, exp, rnd(min,max)\nConstantes: pi, e");
                        rule(ui);small(ui,"Exemplo de enunciado:\nUm carro percorre {d} m em {t} s.\n\nExemplo de alternativa:\n{={d}/{t}} m/s");
                        small(ui,"Use ponto ou vírgula decimal nas fórmulas. Em rnd(min,max), separe os limites por vírgula e use ponto decimal: {={d}*rnd(0.8,1.2)}. Limites iguais reutilizam o mesmo fator no enunciado e nas alternativas desta versão. O sorteio aceita decimais e muda a cada versão. Resultados com até duas casas.");
                    });
                });
            });
        });
    }
    fn build_batch(&mut self) -> bool {
        match generate_with_mode(
            &self.bank,
            &self.setup.plan,
            &self.setup.names,
            self.setup.seed,
            &self.setup.title,
            &self.setup.institution,
            &self.setup.teacher,
            &self.setup.date,
            &self.setup.class_name,
            self.setup.same_questions,
        ) {
            Ok(mut b) => {
                b.logo = self.setup.logo.clone();
                if !self.setup.question_order.is_empty()
                    && !b.apply_question_order(&self.setup.question_order)
                {
                    self.setup.question_order.clear();
                }
                self.batch = Some(b);
                self.exam_preview.invalidate();
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
            page_title(ui, "Criar prova", "Versões diferentes. A mesma facilidade para corrigir.");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
                if primary(ui,"Exportar PDFs").clicked(){
                    if self.batch.is_some()||self.build_batch(){
                        if let Some(base)=rfd::FileDialog::new().set_directory(self.data.join("provas")).pick_folder(){
                            match pdf::export(self.batch.as_ref().unwrap(),&base){Ok(p)=>{let saved=self.remember_batch(&p);self.last_output=Some(p);match saved {Ok(())=>self.notify("Provas e gabarito consolidado salvos em PDF.",false),Err(e)=>self.notify(format!("PDFs salvos. Não foi possível guardar o histórico: {e}"),true)}self.refresh_history();},Err(e)=>self.notify(e,true)}
                        }
                    }
                }
                if secondary(ui,"Conferir prévia").clicked(){if self.batch.is_some()||self.build_batch(){self.exam_preview.open=true;}}
            });
        });
        ui.add_space(PAGE_GAP);
        let w = ui.available_width();
        let mut changed = false;
        let previous_plan = self.setup.plan.clone();
        egui::ScrollArea::vertical().id_salt("exam_setup").auto_shrink([false,false]).show(ui,|ui|{
                ui.set_width(w - ui.spacing().scroll.bar_width - ui.spacing().scroll.bar_inner_margin);
                    card().show(ui,|ui|{
                        ui.label(R::new("1   Dados da prova").size(16.).strong());ui.add_space(10.);
                        let old=(self.setup.title.clone(),self.setup.institution.clone(),self.setup.teacher.clone(),self.setup.class_name.clone(),self.setup.date.clone());
                        field(ui,"Título / disciplina",&mut self.setup.title);field(ui,"Instituição",&mut self.setup.institution);
                        ui.columns(2,|cols|{field(&mut cols[0],"Professor(a)",&mut self.setup.teacher);field(&mut cols[1],"Turma / curso",&mut self.setup.class_name);});
                        field(ui,"Data",&mut self.setup.date);
                        rule(ui);
                        ui.label(R::new("Identidade da instituição").strong().color(TEAL));
                        small(ui,"Logo opcional · PNG ou JPEG. A prova também funciona sem imagem.");
                        if let Some(logo) = &self.setup.logo { show_pictures(ui, std::slice::from_ref(logo)); }
                        ui.horizontal(|ui| {
                            if secondary(ui,if self.setup.logo.is_some(){"Trocar logo"}else{"Adicionar logo"}).clicked(){
                                match pick_picture(){Ok(Some(p))=>{self.setup.logo=Some(p);changed=true;},Err(e)=>self.notify(e,true),_=>{}}
                            }
                            if self.setup.logo.is_some() && ui.button("Remover logo").clicked(){self.setup.logo=None;changed=true;}
                        });
                        changed|=old!=(self.setup.title.clone(),self.setup.institution.clone(),self.setup.teacher.clone(),self.setup.class_name.clone(),self.setup.date.clone());
                    });ui.add_space(12.);
                    let category_panel_width = ui.available_width().min(820.);
                    ui.allocate_ui_with_layout(Vec2::new(category_panel_width,0.),egui::Layout::top_down(egui::Align::Min),|ui|{
                    ui.set_width(category_panel_width);
                    card().show(ui,|ui|{
                        ui.label(R::new("2   Questões por categoria").size(16.).strong());
                        ui.horizontal_wrapped(|ui| {
                            small(ui,"Sorteio:");
                            changed |= ui.selectable_value(&mut self.setup.same_questions,false,"Por aluno").on_hover_text("Questões sorteadas independentemente para cada aluno.").changed();
                            changed |= ui.selectable_value(&mut self.setup.same_questions,true,"Por prova").on_hover_text("Mesmas questões e alternativas; valores calculados por aluno.").changed();
                        });
                        egui::CollapsingHeader::new("Disponibilidade das questões").show(ui, |ui| {
                            small(ui,"Somente prontas e válidas participam. Categorias principais adicionam também as subcategorias, em linhas separadas.");
                            let pending = self.pending_validation.len();
                            small(ui, format!("{} questões no banco · {} rascunhos · {} com erro · {pending} em validação", self.bank.questions.len(), self.bank.questions.iter().filter(|q|q.status==QuestionStatus::Draft).count(), self.bank_errors.len()));
                        });
                        if ui.button("+ Adicionar categoria ou subcategoria").clicked() {
                            self.proof_category_search.clear();
                            self.proof_category_dialog = true;
                            self.proof_category_focus = true;
                        }
                        let len=self.setup.plan.len();let mut mv=None;let mut remove=None;
                        ui.scope(|ui| {
                        ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
                        ui.spacing_mut().scroll.bar_width = 10.;
                        egui::ScrollArea::vertical().id_salt("selected_categories").auto_shrink([false,true]).min_scrolled_height(320.).max_height(320.)
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded).show(ui,|ui|{
                        let category_width=(ui.available_width()-255.).max(120.);
                        ui.scope(|ui| { ui.spacing_mut().interact_size.y=28.; ui.spacing_mut().button_padding=Vec2::new(6.,4.);
                        egui::Grid::new("category_plan").spacing([10.,12.]).show(ui,|ui|{
                            for s in ["Categoria","Disponíveis","Sortear","","",""]{small(ui,s)}ui.end_row();
                            for (i,(cat,n)) in self.setup.plan.iter_mut().enumerate(){
                                let count=self.index.plan_count(cat);
                                let display=proof_pool_label(cat);
                                ui.allocate_ui_with_layout(Vec2::new(category_width,40.),egui::Layout::top_down(egui::Align::Min),|ui|{
                                    ui.set_width(category_width);
                                    ui.add(egui::Label::new(R::new(display.rsplit('/').next().unwrap_or(&display)).strong()).wrap()).on_hover_text(display.replace('/'," / "));
                                    if let Some((parent,_))=display.rsplit_once('/'){ui.add(egui::Label::new(R::new(parent.replace('/'," / ")).size(12.).color(MUTED)).wrap());}
                                });small(ui,count.to_string());let limit=if self.pending_validation.is_empty(){count}else{count.max(*n)};changed|=ui.add_enabled(self.pending_validation.is_empty(),egui::DragValue::new(n).range(0..=limit)).changed();
                                if position_button(ui,true,i>0,"Adiantar categoria").clicked(){mv=Some((i,i-1))}
                                if position_button(ui,false,i+1<len,"Postergar categoria").clicked(){mv=Some((i,i+1))}
                                if ui.button("×").on_hover_text("Retirar categoria desta prova").clicked(){remove=Some(i);}
                                ui.end_row();
                            }
                        });
                        }); });
                        });
                        if let Some((a,b))=mv{self.setup.plan.swap(a,b);changed=true;}
                        if let Some(i)=remove{self.setup.plan.remove(i);changed=true;}
                        if self.setup.plan.is_empty(){small(ui,"Adicione as categorias ou subcategorias que serão usadas nesta prova.");}
                        rule(ui);ui.label(R::new(format!("{} questões por aluno",self.setup.plan.iter().map(|p|p.1).sum::<usize>())).strong().color(TEAL));
                    });
                    });ui.add_space(12.);
                    card().show(ui,|ui|{
                        ui.label(R::new("3   Alunos").size(16.).strong());
                        let grid = self.setup.student_grid.get_or_insert_with(||student_grid::StudentGrid::from_names(&self.setup.names));
                        changed |= grid.show(ui);
                        self.setup.names = grid.names();
                    });ui.add_space(12.);
                    card().show(ui,|ui|{
                        ui.horizontal(|ui|{small(ui,"Identificador do sorteio");changed|=ui.add(egui::DragValue::new(&mut self.setup.seed).range(0..=999_999_999u64)).changed();if ui.button("Novo").clicked(){self.setup.seed=now()%999_999_999;changed=true;}});
                        small(ui,"O mesmo banco, lista e identificador reproduzem as mesmas provas.");
                    });
                    ui.add_space(12.);
                    card().show(ui,|ui|{
                        ui.label(R::new("Prévia em PDF").size(18.).strong());
                        ui.add_space(10.);
                        if let Some(batch)=&self.batch{
                            small(ui,format!("{} alunos · {} questões por versão",batch.students.len(),batch.students[0].questions.len()));
                            ui.add_space(12.);
                            if primary(ui,"Abrir prévia").clicked(){self.exam_preview.open=true;}
                            small(ui,"Confira as páginas, troque de aluno e ajuste a ordem das questões na janela de prévia.");
                        }else{
                            ui.label("Preencha os dados, escolha as quantidades e adicione os alunos.");
                            ui.add_space(12.);
                            small(ui,"Clique em Conferir prévia para abrir as páginas da prova.");
                        }
                    });ui.add_space(12.);
                    egui::Frame::new().fill(PALE).corner_radius(9).inner_margin(18).show(ui,|ui|{
                        ui.label(R::new("O que será salvo").strong().color(TEAL));
                        small(ui,"• Todas as provas em um PDF\n• Gabarito consolidado em PDF");
                    });
        });
        if self.setup.plan != previous_plan {
            self.setup.question_order.clear();
        }
        if changed {
            self.exam_preview.open = false;
            self.exam_preview.invalidate();
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
            page_title(ui, "Arquivos locais", "Seu trabalho organizado, com você.");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if primary(ui, "Exportar banco").clicked() { self.export_bank() }
                if secondary(ui, "Importar banco").clicked() { self.import() }
            });
        });
        ui.add_space(PAGE_GAP);
        egui::ScrollArea::vertical().id_salt("local_files").auto_shrink([false, false]).show(ui, |ui| {
            card().show(ui, |ui| {
                ui.horizontal(|ui| {
                    icon(ui, 2, TEAL, 28.);
                    ui.vertical(|ui| {
                        ui.label(R::new("Banco de questões").size(17.).strong());
                        small(ui, format!("{} questões · banco local · exportação JSON e Moodle XML", self.bank.questions.len()));
                    });
                });
                rule(ui);
                small(ui, "SALVO AUTOMATICAMENTE NESTE COMPUTADOR");
                ui.label(self.data.join("banco.sqlite3").display().to_string());
                ui.add_space(8.);
                if secondary(ui, "Abrir pasta de dados").clicked() { self.open_folder(self.data.clone()) }
                small(ui, "Use Exportar banco para guardar uma cópia ou levar suas questões para outro computador.");
                small(ui, "Importação: JSON deste aplicativo ou XML Moodle (calculatedmulti e multichoice), com imagens PNG/JPG incorporadas. Referências a imagens externas são recusadas com aviso.");
            });
            ui.add_space(PAGE_GAP);
            if let Some(p) = self.last_output.clone() {
                card().show(ui, |ui| {
                    ui.label(R::new("Último lote exportado").size(17.).strong().color(TEAL));
                    ui.label(p.display().to_string());
                    ui.horizontal_wrapped(|ui| {
                        if primary(ui, "Abrir arquivos").clicked() { self.open_folder(p.clone()) }
                        if secondary(ui, "Carregar prévia deste lote").clicked() { self.load_batch(&p) }
                    });
                });
                ui.add_space(PAGE_GAP);
            }
            ui.label(R::new("Provas salvas").size(19.).strong());
            small(ui, "O histórico fica salvo neste computador, incluindo os lotes exportados em outras pastas.");
            ui.horizontal_wrapped(|ui| {
                if secondary(ui, "Abrir registro de lote").clicked() {
                    if let Some(p) = rfd::FileDialog::new().add_filter("Registro do lote", &["json"]).pick_file() {
                        if let Some(parent) = p.parent() { self.load_batch_file(&p, parent.to_path_buf()); }
                    }
                }
                if secondary(ui, "Atualizar").clicked() { self.refresh_history() }
            });
            for entry in self.history.clone() {
                card().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(R::new(&entry.title).strong());
                            small(ui, format!("{} alunos · identificador {}", entry.student_count, entry.seed));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if secondary(ui, "Abrir pasta").clicked() { self.open_folder(entry.folder.clone()) }
                            if secondary(ui, "Ver versões").clicked() { self.load_batch(&entry.folder); }
                        });
                    });
                });
                ui.add_space(12.);
            }
            if self.history.is_empty() {
                card().show(ui, |ui| {
                    ui.label(R::new("Ainda não há provas salvas").strong());
                    small(ui, "Crie uma prova e exporte os PDFs para encontrá-la aqui.");
                    if secondary(ui, "Criar prova").clicked() { self.navigate(2); }
                });
            }
        });
    }
    fn load_batch(&mut self, p: &std::path::Path) {
        let result = export_history::read(&self.data, p);
        self.restore_batch(result, p.to_path_buf());
    }
    fn load_batch_file(&mut self, p: &std::path::Path, folder: PathBuf) {
        let result = std::fs::read(p)
            .map_err(|e| e.to_string())
            .and_then(|b| serde_json::from_slice::<Batch>(&b).map_err(|e| e.to_string()));
        self.restore_batch(result, folder);
    }
    fn restore_batch(&mut self, result: Result<Batch, String>, folder: PathBuf) {
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
                self.setup.same_questions = b.same_questions;
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
                self.setup.question_order = b.question_order.clone();
                self.setup.title = b.title.clone();
                self.setup.seed = b.seed;
                self.setup.student_grid = None;
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
                self.setup.logo = b.logo.clone();
                self.batch = Some(b);
                self.exam_preview.invalidate();
                self.exam_preview.open = true;
                self.student_preview = 0;
                self.last_output = Some(folder);
                self.page = 2;
                self.notify(
                    "Prévia carregada com os valores e alternativas originais.",
                    false,
                );
            }
            Ok(_) => self.notify("Registro vazio ou incompatível.", true),
            Err(e) => self.notify(format!("Registro inválido: {e}"), true),
        }
    }
    fn start_update(&mut self, ctx: &egui::Context) {
        if self.dirty {
            self.notify(
                "Salve ou descarte as alterações da questão antes de atualizar.",
                true,
            );
            return;
        }
        self.updater.install(ctx);
    }
    fn updates_page(&mut self, ui: &mut egui::Ui) {
        page_title(ui, "Sobre e atualizações", "Novas versões, com seus arquivos sempre no computador.");
        ui.add_space(PAGE_GAP);
        egui::ScrollArea::vertical().id_salt("about_updates").auto_shrink([false, false]).show(ui, |ui| {
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
                    field(ui, "Repositório ou link de releases", &mut self.update_repository);
                    ui.checkbox(&mut self.update_automatic, "Verificar ao iniciar e periodicamente");
                    if primary(ui, "Salvar preferências").clicked() {
                        if let Err(e) = self.updater.save(&self.data, &self.update_repository, self.update_automatic) { self.updater.message = e; }
                    }
                });
                ui.add_space(8.);
                if ui.add_enabled(!busy, egui::Button::new("Verificar agora")).clicked() { self.updater.check(ui.ctx()); }
                if busy { ui.spinner(); }
                ui.label(&self.updater.message);
                if self.updater.available.is_some() {
                    ui.add_enabled_ui(!busy, |ui| { if primary(ui, "Baixar e instalar").clicked() { self.start_update(ui.ctx()); } });
                }
                if let Ok(repo) = updater::repository(&self.updater.settings.repository) {
                    ui.horizontal(|ui| {
                        ui.hyperlink_to("Código aberto", format!("https://github.com/{repo}"));
                        ui.hyperlink_to("Todas as releases", format!("https://github.com/{repo}/releases"));
                    });
                }
                rule(ui);
                small(ui, "Baixe e instale por aqui. Após sua autorização no Windows, o aplicativo fecha para concluir a atualização. Na versão portátil, o executável é substituído e a pasta data é preservada.");
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
        if let Some(batch) = &mut self.batch {
            if self
                .exam_preview
                .show(ctx, batch, &mut self.student_preview)
            {
                self.setup.question_order = batch.question_order.clone();
            }
        }
        self.import_dialog(ctx);
        self.category_delete_dialog(ctx);
        if self.proof_category_dialog {
            let (selected, close) = category_view::proof_category_picker(
                ctx,
                &mut self.proof_category_search,
                &mut self.proof_category_focus,
                &self.index,
                &self.setup.plan,
                !self.pending_validation.is_empty(),
            );
            if let Some(cat) = selected {
                self.index.add_category_tree(&cat, &mut self.setup.plan);
                self.setup.question_order.clear();
                self.batch = None;
                self.proof_category_dialog = false;
            }
            if close {
                self.proof_category_dialog = false;
            }
        }
        if self.export_dialog.is_some() {
            let mut choice = None;
            let mut close = false;
            let response = egui::Modal::new(egui::Id::new("export_bank_format")).show(ctx, |ui| {
                ui.set_width(430.);
                ui.heading("Exportar questões");
                small(ui, format!("{} questões na exportação", self.export_dialog.as_ref().unwrap().questions.len()));
                ui.label("JSON preserva fórmulas, rascunhos e questões com erro para continuar editando no Test Craft.");
                if primary(ui, "Salvar JSON").clicked() {choice = Some(false);}
                rule(ui);
                ui.label("Moodle XML inclui somente questões prontas e válidas, com categorias, imagens e respostas corretas.");
                small(ui, "As questões calculadas são convertidas em uma versão com valores fixos. Para sortear novas versões, use o Test Craft.");
                if primary(ui, "Salvar Moodle XML").clicked() {choice = Some(true);}
                if self.status_error {ui.colored_label(ERROR, &self.status);}
                if ui.button("Cancelar").clicked() {close = true;}
            });
            if let Some(xml) = choice {
                let bank = self.export_dialog.as_ref().unwrap();
                let dialog = rfd::FileDialog::new();
                let path = if xml {
                    dialog
                        .add_filter("Moodle XML", &["xml"])
                        .set_file_name("questoes_moodle.xml")
                        .save_file()
                } else {
                    dialog
                        .add_filter("Banco JSON", &["json"])
                        .set_file_name("banco_de_questoes.json")
                        .save_file()
                };
                if let Some(path) = path {
                    let result = if xml {
                        moodle::bank_xml(bank, 2026).and_then(|(content, count)| {
                            std::fs::write(&path, content).map_err(|e| e.to_string())?;
                            Ok(format!("Moodle XML salvo: {count} questões; {} rascunhos ou questões com erro excluídos.", bank.questions.len() - count))
                        })
                    } else {
                        save_bank(&path, bank).map(|()| "Banco exportado em JSON.".into())
                    };
                    match result {
                        Ok(message) => {
                            self.notify(message, false);
                            close = true;
                        }
                        Err(e) => self.notify(e, true),
                    }
                }
            }
            if close || response.should_close() {
                self.export_dialog = None;
            }
        }
        if let Some(copy) = self.transfer {
            egui::Modal::new(egui::Id::new("transfer_questions")).show(ctx, |ui| {
                ui.set_width(430.);
                ui.heading(if copy {
                    "Copiar questões"
                } else {
                    "Mover questões"
                });
                ui.label(format!(
                    "{} questões selecionadas, incluindo seleções de outros filtros.",
                    self.checked.len()
                ));
                small(ui, "Categoria de destino");
                category_dropdown(
                    ui,
                    "transfer_destination",
                    &mut self.transfer_destination,
                    &self.index.categories,
                    false,
                    &self.index.category_counts,
                );
                if !copy {
                    small(
                        ui,
                        "As questões sairão da categoria atual. As categorias serão preservadas.",
                    );
                }
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            !self.transfer_destination.is_empty(),
                            egui::Button::new(if copy {
                                "Confirmar cópia"
                            } else {
                                "Confirmar mudança"
                            }),
                        )
                        .clicked()
                    {
                        let old = self.bank.clone();
                        match transfer_questions(
                            &mut self.bank,
                            &self.checked,
                            &self.transfer_destination,
                            copy,
                        ) {
                            Ok(count) => {
                                if self.persist(&old) {
                                    self.sync_plan();
                                    self.category = "Todas".into();
                                    self.search.clear();
                                    self.checked.clear();
                                    self.transfer = None;
                                    self.notify(
                                        format!(
                                            "{count} questões {} e salvas.",
                                            if copy { "copiadas" } else { "movidas" }
                                        ),
                                        false,
                                    );
                                } else {
                                    self.bank = old;
                                }
                            }
                            Err(e) => self.notify(e, true),
                        }
                    }
                    if ui.button("Cancelar").clicked() {
                        self.transfer = None;
                    }
                });
                if self.status_error {
                    ui.colored_label(ERROR, &self.status);
                }
            });
        }
        if let Some(ids) = self.batch_delete.clone() {
            egui::Modal::new(egui::Id::new("delete_selection")).show(ctx, |ui| {
                ui.set_width(420.);
                ui.heading(format!("Apagar {} questões?", ids.len()));
                ui.label("A seleção inclui questões marcadas em outros filtros. Esta ação remove as questões do banco local.");
                small(ui, "As categorias e as provas já exportadas serão preservadas.");
                egui::ScrollArea::vertical().max_height(160.).show(ui, |ui| {
                    for q in self.bank.questions.iter().filter(|q| ids.contains(&q.id)) {
                        ui.label(format!("{} · {}", q.title, q.category));
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("Cancelar").clicked() { self.batch_delete = None; }
                    if primary(ui, "Apagar seleção").clicked() {
                        let old = self.bank.clone();
                        self.bank.category_labels = categories(&self.bank);
                        self.bank.questions.retain(|q| !ids.contains(&q.id));
                        if self.persist(&old) {
                            self.checked.retain(|id| !ids.contains(id));
                            if self.selected.is_some_and(|id| ids.contains(&id)) { self.selected = self.bank.questions.first().map(|q| q.id); }
                            self.sync_plan();
                            self.batch_delete = None;
                            self.notify(format!("{} questões apagadas.", ids.len()), false);
                        } else { self.bank = old; }
                    }
                });
                if self.status_error { ui.colored_label(ERROR, &self.status); }
            });
        }
        if self.category_dialog {
            egui::Modal::new(egui::Id::new("new_category_dialog")).show(ctx, |ui| {
                ui.set_width(670.);
                egui::ScrollArea::vertical().id_salt("category_dialog_body").max_height((ctx.screen_rect().height()-100.).max(300.)).show(ui,|ui| {
                ui.heading("Configurar categorias");
                small(ui,format!("{} categorias · {} questões",self.index.categories.len(),self.bank.questions.len()));
                ui.add_space(8.);
                ui.add(egui::TextEdit::singleline(&mut self.category_search).hint_text("Buscar categoria...").desired_width(f32::INFINITY));
                let choices:Vec<_>=self.index.categories.iter().filter(|p|p.to_lowercase().contains(&self.category_search.to_lowercase())).cloned().collect();
                egui::ScrollArea::vertical().id_salt("manage_categories").max_height(210.).show(ui,|ui| {
                    for path in choices {
                        let count=self.index.category_counts.get(&path).copied().unwrap_or(0);
                        ui.horizontal(|ui| {
                            let mut checked=self.category_checked.contains(&path);
                            if ui.checkbox(&mut checked, "").changed() { if checked {self.category_checked.insert(path.clone());} else {self.category_checked.remove(&path);} }
                            if ui.selectable_label(self.category_edit.as_ref()==Some(&path),format!("{} ({count})",path.replace('/'," / ")).as_str()).clicked(){self.edit_category_path(path.clone());}
                        });
                    }
                    if self.index.categories.is_empty(){small(ui,"Crie a primeira categoria abaixo.");}
                });
                ui.horizontal(|ui| {
                    if ui.button("Nova categoria").clicked(){self.category_edit=None;self.category_name.clear();self.category_parent.clear();self.category_error=None;}
                    if let Some(path)=self.category_edit.clone(){
                        if ui.button(if self.category_for_editor {"Usar na questão"} else {"Filtrar no banco"}).clicked(){
                            if self.category_for_editor{self.draft.category=path.clone();self.dirty=true;self.validation=None; self.validation_example=None;}else{self.category=path.clone();self.search.clear();}
                            self.category_dialog=false;
                        }
                        if ui.button("Excluir categoria...").clicked(){
                            self.category_delete=Some(path);
                            self.category_delete_destination.clear();
                            self.category_dialog=false;
                            self.category_error=None;
                        }
                    }
                });
                self.category_bulk_controls(ui);
                rule(ui);
                ui.label(R::new(if self.category_edit.is_some(){"Editar nome ou categoria principal"}else{"Criar categoria"}).strong().color(TEAL));
                small(ui, "Organize assuntos em categorias e subcategorias.");
                if let Some(old) = &self.category_edit { ui.label(format!("Editando: {}. As questões e subcategorias acompanharão a alteração.", old.replace('/', " / "))); }
                ui.add_space(12.);
                field(ui, "Nome", &mut self.category_name);
                small(ui, "Dentro de (opcional)");
                egui::ComboBox::from_id_salt("category_parent")
                    .selected_text(if self.category_parent.is_empty() {
                        "Nenhuma · categoria principal".to_owned()
                    } else {
                        format!("{} ({})", self.category_parent, self.index.category_counts.get(&self.category_parent).copied().unwrap_or(0))
                    })
                    .width(ui.available_width())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.category_parent,
                            String::new(),
                            "Nenhuma · categoria principal",
                        );
                        let choices: Vec<_> = self.index.categories.clone().into_iter().filter(|c| self.category_edit.as_ref().is_none_or(|old| !category_matches(c, old))).collect();
                        category_options(ui, &mut self.category_parent, &choices, &self.index.category_counts);
                    });
                ui.add_space(8.);
                if !self.category_name.trim().is_empty() {
                    small(
                        ui,
                        format!(
                            "Caminho: {}{}{}",
                            self.category_parent,
                            if self.category_parent.is_empty() {
                                ""
                            } else {
                                " / "
                            },
                            self.category_name.trim()
                        ),
                    );
                }
                if let Some(error) = &self.category_error {
                    ui.colored_label(ERROR, error);
                }
                ui.add_space(12.);
                ui.horizontal(|ui| {
                    if primary(ui, if self.category_edit.is_some() {"Salvar categoria"} else {"Criar categoria"}).clicked() {
                        let editing=self.category_edit.is_some();
                        let old = self.bank.clone();
                        let result = if let Some(source) = &self.category_edit {
                            rename_category(&mut self.bank, source, &self.category_name, &self.category_parent)
                        } else { new_category(
                            &self.category_name,
                            &self.category_parent,
                            &self.index.categories,
                        ) };
                        match result {
                            Err(e) => self.category_error = Some(e),
                            Ok(path) => {
                                if self.category_edit.is_none() { self.bank.category_labels.push(path.clone()); }
                                if self.persist(&old) {
                                    if let Some(source) = &self.category_edit {
                                        self.draft.category = renamed_path(&self.draft.category, source, &path);
                                        for (category, _) in &mut self.setup.plan { *category = renamed_proof_pool(category, source, &path); }
                                    }
                                    self.sync_plan();
                                    if self.category_for_editor {
                                        self.draft.category = path.clone();
                                        self.dirty = true;
                                        self.validation=None; self.validation_example=None;
                                    } else {
                                        // Creating a category must not hide the rest of the bank.
                                        self.category = "Todas".into();
                                        self.search.clear();
                                    }
                                    self.edit_category_path(path.clone());
                                    self.notify(format!("Categoria {} e salva: {}. Todas as categorias continuam disponíveis.", if editing {"atualizada"} else {"criada"}, self.category_name.trim()), false);
                                } else {
                                    self.bank = old;
                                    self.category_error = Some(self.status.clone());
                                }
                            }
                        }
                    }
                    if secondary(ui, "Fechar").clicked() {
                        self.category_dialog = false;
                    }
                });
                });
            });
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.category_dialog = false;
            }
        }
        if self.expanded_preview {
            let question = if self.page == 1 {
                Some(self.draft.clone())
            } else {
                self.bank
                    .questions
                    .iter()
                    .find(|q| Some(q.id) == self.selected)
                    .cloned()
            };
            let mut open = true;
            egui::Window::new("Prévia ampliada")
                .open(&mut open)
                .collapsible(false)
                .default_width(760.)
                .default_height(620.)
                .resizable(true)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                            if let Some(q) = &question {
                                ui.heading(&q.title);
                                small(ui, q.category.replace('/', " / "));
                                rule(ui);
                                let example = if self.page == 1 {
                                    self.validation_example.as_ref()
                                } else {
                                    self.bank_examples.get(&q.id)
                                };
                                Self::show_question_preview(ui, q, self.preview_seed, example);
                                ui.add_space(16.);
                                if example.is_none() && secondary(ui, "Sortear valores").clicked() {
                                    self.preview_seed += 1;
                                }
                            } else {
                                ui.label("Selecione uma questão no banco.");
                            }
                        });
                });
            self.expanded_preview = open && !ctx.input(|i| i.key_pressed(egui::Key::Escape));
        }

        if self.updater.offer {
            if let Some(release) = self.updater.available.clone() {
                let mut open = true;
                egui::Window::new("Nova versão disponível")
                    .open(&mut open).collapsible(false).resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO).default_width(420.)
                    .show(ctx, |ui| {
                        ui.label(R::new(format!("Test Craft {} está disponível", release.version)).size(20.).strong().color(TEAL));
                        ui.label(format!("Versão instalada: {}", updater::VERSION));
                        ui.label("O aplicativo baixa e verifica o pacote. Depois, o Windows solicita autorização quando necessária para instalar. Salve suas edições antes de continuar. Seus dados são preservados.");
                        ui.horizontal(|ui| {
                            if primary(ui, "Baixar e instalar").clicked() {
                                self.start_update(ctx);
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
                            if self.persist(&old) {
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
        if self.updater.installing() {
            egui::Modal::new(egui::Id::new("installing_update")).show(ctx, |ui| {
                ui.set_width(420.);
                ui.heading("Atualizando Test Craft");
                ui.horizontal(|ui| { ui.spinner(); ui.label(&self.updater.message); });
                small(ui,"Aguarde o download e confirme a autorização do Windows. O aplicativo fechará para instalar; depois, abra-o novamente.");
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
        while let Ok(result) = self.validator.results.try_recv() {
            if result.generation == self.validation_generation {
                self.index.invalidate_filter();
                self.pending_validation.remove(&result.id);
                if let Some(example) = result.example {
                    self.bank_examples.insert(result.id, example);
                } else {
                    self.bank_examples.remove(&result.id);
                }
                if let Some(error) = result.error {
                    self.bank_errors.insert(result.id, error);
                } else {
                    self.bank_errors.remove(&result.id);
                    if let Some(&position) = self.index.positions.get(&result.id) {
                        let q = &self.bank.questions[position];
                        if q.status == QuestionStatus::Ready {
                            self.index.record_ready(q);
                        }
                    }
                }
            }
        }
        if self.screenshot.is_none() {
            self.updater.tick(ctx);
            if self.updater.close_requested {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
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
            if self.frames >= 3 && !self.screenshot_requested {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                self.screenshot_requested = true;
            }
            if self.screenshot_started.elapsed() > std::time::Duration::from_secs(15) {
                eprintln!(
                    "Screenshot: captura não recebida em 15 segundos; encerrando verificação."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }
        self.sidebar(ctx);
        egui::TopBottomPanel::top("topbar")
            .exact_height(60.)
            .frame(
                egui::Frame::new()
                    .fill(C::WHITE)
                    .inner_margin(Margin::symmetric(28, 11)),
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
                    if self.updater.check_failed {
                        if ui.small_button("Atualizações: verificar conexão").on_hover_text(&self.updater.message).clicked() { self.navigate(4); }
                    } else if self.updater.busy() {
                        ui.spinner();
                        small(ui, &self.updater.message);
                    } else if let Some(release) = &self.updater.available {
                        if ui.small_button(format!("Instalar versão {}", release.version)).clicked() { self.navigate(4); }
                    }
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
            if let Ok(bytes) = serde_json::to_vec(&self.sidebar_collapsed) {
                let _ = std::fs::write(self.data.join("menu-recolhido.json"), bytes);
            }
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
            .with_title("Test Craft · Provas individualizadas")
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../assets/test-craft.png"))
                    .expect("Ícone incorporado válido"),
            ),
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
