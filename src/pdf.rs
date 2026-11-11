use crate::model::*;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
// A4 retained; user-requested compact layout uses 1.5 cm margins.
pub const WIDTH: f64 = 595.2755905511812;
pub const HEIGHT: f64 = 841.8897637795276;
pub const MARGIN: f64 = 42.51968503937008;
// MoodTest.py add_blank_answer_tables, source offset 0x7e27 (block at 0x7f2f).
const ANSWER_TABLE_BLOCK: usize = 15;

fn win(c: char) -> u8 {
    match c {
        '€' => 128,
        '–' => 150,
        '—' => 151,
        '‘' => 145,
        '’' => 146,
        '“' => 147,
        '”' => 148,
        '•' => 149,
        '…' => 133,
        'Ω' => b'O',
        'ω' => b'w',
        '−' => b'-',
        _ if c as u32 <= 255 => c as u8,
        _ => b'?',
    }
}
fn literal(s: &str) -> String {
    let mut t = String::from("(");
    for c in s.chars() {
        let b = win(c);
        if b == b'(' || b == b')' || b == b'\\' {
            t.push('\\');
            t.push(b as char)
        } else if b < 32 || b > 126 {
            t.push_str(&format!("\\{:03o}", b))
        } else {
            t.push(b as char)
        }
    }
    t.push(')');
    t
}
fn symbol(c: char) -> Option<char> {
    let greek = "ΑΒΓΔΕΖΗΘΙΚΛΜΝΞΟΠΡΣΤΥΦΧΨΩαβγδεζηθικλμνξοπρστυφχψω";
    let codes = "ABGDEZHQIKLMNXOPRSTUFCYWabgdezhqiklmnxoprstufcyw";
    greek
        .chars()
        .position(|v| v == c)
        .and_then(|i| codes.chars().nth(i))
        .or_else(|| match c {
            '∞' => Some('¥'),
            '≤' => Some('£'),
            '≥' => Some('³'),
            '√' => Some('Ö'),
            '≠' => Some('¹'),
            '≈' => Some('»'),
            _ => None,
        })
}
fn text_commands(t: &str, size: f64, bold: bool) -> String {
    let mut out = String::new();
    let mut chunk = String::new();
    let mut sym = false;
    for c in t.chars() {
        let mapped = symbol(c);
        let next = mapped.is_some();
        if !chunk.is_empty() && next != sym {
            out.push_str(&format!(
                "/{} {size} Tf {} Tj ",
                if sym {
                    "F3"
                } else if bold {
                    "F2"
                } else {
                    "F1"
                },
                literal(&chunk)
            ));
            chunk.clear();
        }
        sym = next;
        chunk.push(mapped.unwrap_or(c));
    }
    if !chunk.is_empty() {
        out.push_str(&format!(
            "/{} {size} Tf {} Tj ",
            if sym {
                "F3"
            } else if bold {
                "F2"
            } else {
                "F1"
            },
            literal(&chunk)
        ));
    }
    out
}
fn text_width(s: &str, size: f64, bold: bool) -> f64 {
    const NORMAL: [u16; 95] = [
        278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556,
        556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556, 1015, 667, 667, 722,
        722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, 667, 778, 722, 667, 611, 722,
        667, 944, 667, 667, 611, 278, 278, 278, 469, 556, 333, 556, 556, 500, 556, 556, 278, 556,
        556, 222, 222, 500, 222, 833, 556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500,
        500, 334, 260, 334, 584,
    ];
    const BOLD: [u16; 95] = [
        278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556,
        556, 556, 556, 556, 556, 556, 556, 556, 333, 333, 584, 584, 584, 611, 975, 722, 722, 722,
        722, 667, 611, 778, 722, 278, 556, 722, 611, 833, 722, 778, 667, 778, 722, 667, 611, 722,
        667, 944, 667, 667, 611, 333, 278, 333, 584, 556, 333, 556, 611, 556, 611, 556, 333, 611,
        611, 278, 278, 556, 278, 889, 611, 611, 611, 611, 389, 556, 333, 611, 556, 778, 556, 556,
        500, 389, 280, 389, 584,
    ];
    s.chars()
        .map(|c| {
            let base = match c {
                'á' | 'à' | 'ã' | 'â' | 'ä' => 'a',
                'é' | 'è' | 'ê' | 'ë' => 'e',
                'í' | 'ì' | 'î' | 'ï' => 'i',
                'ó' | 'ò' | 'õ' | 'ô' | 'ö' => 'o',
                'ú' | 'ù' | 'û' | 'ü' => 'u',
                'ç' => 'c',
                'Á' | 'À' | 'Ã' | 'Â' | 'Ä' => 'A',
                'É' | 'È' | 'Ê' | 'Ë' => 'E',
                'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
                'Ó' | 'Ò' | 'Õ' | 'Ô' | 'Ö' => 'O',
                'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
                'Ç' => 'C',
                _ => c,
            };
            if (' '..='~').contains(&base) {
                (if bold { BOLD } else { NORMAL })[base as usize - 32] as f64 * size / 1000.
            } else {
                size
            }
        })
        .sum()
}
struct PdfImage {
    width: u32,
    height: u32,
    data: Vec<u8>,
}
fn wrapped_lines(t: &str, size: f64, bold: bool) -> Vec<String> {
    wrapped_lines_width(t, size, bold, WIDTH - 2. * MARGIN)
}
fn wrapped_lines_width(t: &str, size: f64, bold: bool, max: f64) -> Vec<String> {
    let mut lines = vec![];
    for para in t.split('\n') {
        let mut line = String::new();
        for word in para.split_whitespace() {
            let candidate = if line.is_empty() {
                word.to_owned()
            } else {
                format!("{line} {word}")
            };
            if text_width(&candidate, size, bold) <= max {
                line = candidate;
                continue;
            }
            if !line.is_empty() {
                lines.push(std::mem::take(&mut line));
            }
            for c in word.chars() {
                if text_width(&format!("{line}{c}"), size, bold) > max && !line.is_empty() {
                    lines.push(std::mem::take(&mut line));
                }
                line.push(c);
            }
        }
        lines.push(line);
    }
    lines
}
pub struct Pdf {
    pictures: Vec<PdfImage>,
    pages: Vec<String>,
    page: String,
    y: f64,
    title: String,
    name: String,
    number: usize,
}
impl Pdf {
    pub fn new(title: &str, name: &str) -> Self {
        let mut p = Self {
            pictures: vec![],
            pages: vec![],
            page: String::new(),
            y: HEIGHT - MARGIN,
            title: title.into(),
            name: name.into(),
            number: 0,
        };
        p.begin();
        p
    }
    fn begin(&mut self) {
        self.number += 1;
        self.y = HEIGHT - MARGIN;
        self.page.push_str("0.10 0.20 0.19 rg\n");
        self.line(
            &self.title.clone(),
            if self.number == 1 { 16. } else { 11. },
            true,
        );
        self.gap(3.);
        self.line(&format!("Aluno(a): {}", self.name), 10., false);
        self.gap(4.);
        self.page.push_str(&format!(
            "0.22 0.38 0.34 RG 1 w {MARGIN} {} m {} {} l S\n",
            self.y,
            WIDTH - MARGIN,
            self.y
        ));
        let footer = format!("Página {}", self.number);
        let x = WIDTH - MARGIN - text_width(&footer, 8., false);
        self.page.push_str(&format!(
            "0.35 g BT 1 0 0 1 {x:.3} 24 Tm {} ET 0 g\n",
            text_commands(&footer, 8., false)
        ));
        self.y -= 12.;
    }
    fn draw(&mut self, t: &str, size: f64, bold: bool) {
        self.page.push_str(&format!(
            "BT 1 0 0 1 {MARGIN:.3} {:.3} Tm {} ET\n",
            self.y,
            text_commands(t, size, bold)
        ));
        self.y -= size * 1.22;
    }
    fn next(&mut self) {
        self.pages.push(std::mem::take(&mut self.page));
        self.begin();
    }
    pub fn line(&mut self, t: &str, size: f64, bold: bool) {
        for line in wrapped_lines(t, size, bold) {
            if self.y < MARGIN + size {
                self.next();
            }
            self.draw(&line, size, bold);
        }
    }
    pub fn gap(&mut self, n: f64) {
        self.y -= n;
    }
    pub fn ensure(&mut self, height: f64) {
        if self.y - height < MARGIN {
            self.next()
        }
    }
    fn blank_answer_tables(&mut self, count: usize) {
        // Keep each header and its empty writing row together on one page.
        self.ensure(if count > 0 { 78. } else { 20. });
        self.line("Gabarito (preencha)", 12., true);
        self.gap(6.);
        let cell_width = (WIDTH - 2. * MARGIN) / ANSWER_TABLE_BLOCK as f64;
        for start in (1..=count).step_by(ANSWER_TABLE_BLOCK) {
            self.ensure(56.);
            let end = (start + ANSWER_TABLE_BLOCK - 1).min(count);
            let columns = end - start + 1;
            let width = cell_width * columns as f64;
            let top = self.y;
            let middle = top - 22.;
            let bottom = top - 48.;
            self.page.push_str(&format!(
                "q 0.827451 g {MARGIN:.3} {middle:.3} {width:.3} 22 re f 0 G 0 g 0.75 w\n"
            ));
            for y in [top, middle, bottom] {
                self.page.push_str(&format!(
                    "{MARGIN:.3} {y:.3} m {:.3} {y:.3} l S\n",
                    MARGIN + width
                ));
            }
            for column in 0..=columns {
                let x = MARGIN + column as f64 * cell_width;
                self.page
                    .push_str(&format!("{x:.3} {bottom:.3} m {x:.3} {top:.3} l S\n"));
            }
            for (column, index) in (start..=end).enumerate() {
                let label = format!("Q{index:02}");
                let x = MARGIN
                    + column as f64 * cell_width
                    + (cell_width - text_width(&label, 10., true)) / 2.;
                self.page.push_str(&format!(
                    "BT 1 0 0 1 {x:.3} {:.3} Tm {} ET\n",
                    top - 14.,
                    text_commands(&label, 10., true)
                ));
            }
            self.page.push_str("Q\n");
            self.y = bottom - 8.;
        }
        self.gap(8.);
    }
    pub fn picture(&mut self, pic: &Picture) {
        let height = picture_height(pic);
        self.ensure(height + 4.);
        self.y -= self.image_at(pic, MARGIN, self.y, 350., 220.) + 12.;
    }
    fn image_at(&mut self, pic: &Picture, x: f64, top: f64, max_w: f64, max_h: f64) -> f64 {
        use std::io::Write;
        if let Ok(img) = image::load_from_memory(&pic.data) {
            // Composite transparent logos on white rather than turning them black.
            let mut rgba = img.thumbnail(1280, 1280).to_rgba8();
            for pixel in rgba.pixels_mut() {
                let alpha = pixel[3] as u32;
                for channel in 0..3 {
                    pixel[channel] =
                        ((pixel[channel] as u32 * alpha + 255 * (255 - alpha)) / 255) as u8;
                }
                pixel[3] = 255;
            }
            let rgb = image::DynamicImage::ImageRgba8(rgba).to_rgb8();
            let (w, h) = rgb.dimensions();
            let scale = (max_w / w as f64).min(max_h / h as f64).min(1.);
            let dw = w as f64 * scale;
            let dh = h as f64 * scale;
            let id = self.pictures.len();
            let mut encoder =
                flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            if encoder.write_all(rgb.as_raw()).is_ok() {
                if let Ok(data) = encoder.finish() {
                    self.page.push_str(&format!(
                        "q {dw} 0 0 {dh} {x} {} cm /Im{id} Do Q\n",
                        top - dh
                    ));
                    self.pictures.push(PdfImage {
                        width: w,
                        height: h,
                        data,
                    });
                    return dh;
                }
            }
        }
        0.
    }
    pub fn append(&mut self, mut other: Pdf) {
        if !self.page.is_empty() {
            self.pages.push(std::mem::take(&mut self.page));
        }
        let offset = self.pictures.len();
        for i in (0..other.pictures.len()).rev() {
            let from = format!("/Im{i} Do");
            let to = format!("/Im{} Do", i + offset);
            other.page = other.page.replace(&from, &to);
            for page in &mut other.pages {
                *page = page.replace(&from, &to);
            }
        }
        self.pictures.append(&mut other.pictures);
        self.pages.extend(other.finish_pages());
    }
    fn finish_pages(mut self) -> Vec<String> {
        if !self.page.is_empty() {
            self.pages.push(self.page)
        }
        self.pages
    }
    pub fn bytes(mut self) -> Vec<u8> {
        let pictures = std::mem::take(&mut self.pictures);
        let pages = self.finish_pages();
        let mut objects: Vec<Vec<u8>> = vec![];
        objects.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());
        let kids = (0..pages.len())
            .map(|i| format!("{} 0 R", 5 + i * 2))
            .collect::<Vec<_>>()
            .join(" ");
        objects.push(
            format!("<< /Type /Pages /Count {} /Kids [{}] >>", pages.len(), kids).into_bytes(),
        );
        objects.push(
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
                .to_vec(),
        );
        objects.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>".to_vec());
        let image_start = 5 + pages.len() * 2;
        let symbol_id = image_start + pictures.len();
        let resources = pictures
            .iter()
            .enumerate()
            .map(|(i, _)| format!("/Im{i} {} 0 R", image_start + i))
            .collect::<Vec<_>>()
            .join(" ");
        for (i, page) in pages.iter().enumerate() {
            objects.push(format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {WIDTH} {HEIGHT}] /Resources << /Font << /F1 3 0 R /F2 4 0 R /F3 {symbol_id} 0 R >> /XObject << {resources} >> >> /Contents {} 0 R >>",6+i*2).into_bytes());
            objects.push(
                format!("<< /Length {} >>\nstream\n{}\nendstream", page.len(), page).into_bytes(),
            );
        }
        for pic in pictures {
            let mut obj=format!("<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode /Length {} >>\nstream\n",pic.width,pic.height,pic.data.len()).into_bytes();
            obj.extend(pic.data);
            obj.extend(b"\nendstream");
            objects.push(obj);
        }
        objects.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Symbol >>".to_vec());
        let mut out = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n".to_vec();
        let mut offsets = vec![0];
        for (i, obj) in objects.iter().enumerate() {
            offsets.push(out.len());
            out.extend(format!("{} 0 obj\n", i + 1).as_bytes());
            out.extend(obj);
            out.extend(b"\nendobj\n");
        }
        let start = out.len();
        out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len()).as_bytes());
        for offset in offsets.into_iter().skip(1) {
            out.extend(format!("{offset:010} 00000 n \n").as_bytes())
        }
        out.extend(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{start}\n%%EOF\n",
                objects.len() + 1
            )
            .as_bytes(),
        );
        out
    }
}
pub fn letters(q: &Instance) -> String {
    let letters = q
        .answers
        .iter()
        .enumerate()
        .filter(|(_, a)| a.correct)
        .map(|(i, _)| ((b'a' + i as u8) as char).to_string())
        .collect::<Vec<_>>()
        .join("/");
    // MoodTest.py indices_para_letras, source offset 0xd0f7.
    if letters.is_empty() {
        "-".into()
    } else {
        letters
    }
}
fn exam(batch: &Batch, s: &StudentExam) -> Pdf {
    let mut p = Pdf::new(&batch.title, &s.name);
    // Replace the generic first-page heading with a print-friendly exam header.
    p.page.clear();
    p.y = HEIGHT - MARGIN;
    let top = p.y;
    let logo_height = batch
        .logo
        .as_ref()
        .map(|logo| p.image_at(logo, WIDTH - MARGIN - 80., top, 80., 65.))
        .unwrap_or(0.);
    let text_max = WIDTH - 2. * MARGIN - if batch.logo.is_some() { 100. } else { 0. };
    p.page.push_str("0.10 0.20 0.19 rg\n");
    for (text, size, bold) in [(&batch.institution, 10., false), (&batch.title, 18., true)] {
        if !text.trim().is_empty() {
            for line in wrapped_lines_width(text, size, bold, text_max) {
                p.ensure(size * 1.22);
                p.draw(&line, size, bold);
            }
            p.gap(6.);
        }
    }
    p.y = p.y.min(top - logo_height - 8.);
    p.page.push_str(&format!(
        "0.22 0.44 0.38 RG 2 w {MARGIN} {} m {} {} l S 0 g\n",
        p.y,
        WIDTH - MARGIN,
        p.y
    ));
    p.gap(14.);
    p.line(&format!("Aluno(a): {}", s.name), 11., true);
    p.gap(5.);
    for t in [
        &format!(
            "Professor(a): {}   |   Turma: {}",
            batch.teacher, batch.class_name
        ),
        &format!("Data: {}   |   Identificador: {}", batch.date, batch.seed),
    ] {
        if !t.trim().is_empty() {
            p.line(t, 10., false)
        }
    }
    p.page.push_str(&format!(
        "0.35 g BT 1 0 0 1 {} 24 Tm {} ET 0 g\n",
        WIDTH - MARGIN - text_width("Página 1", 8., false),
        text_commands("Página 1", 8., false)
    ));
    p.gap(5.);
    p.line(
        "Responda às questões abaixo. Assinale as alternativas corretas.",
        9.,
        false,
    );
    p.gap(5.);
    p.blank_answer_tables(s.questions.len());
    p.gap(4.);
    for (i, q) in s.questions.iter().enumerate() {
        let title = format!("{}. {}", i + 1, q.text);
        let height = wrapped_lines(&title, 10.5, true).len() as f64 * 10.5 * 1.22
            + q.answers
                .iter()
                .enumerate()
                .map(|(j, a)| {
                    wrapped_lines(
                        &format!("   {}) {}", (b'a' + j as u8) as char, a.text),
                        10.5,
                        false,
                    )
                    .len() as f64
                        * 10.5
                        * 1.22
                        + a.pictures.iter().map(picture_height).sum::<f64>()
                })
                .sum::<f64>()
            + q.pictures.iter().map(picture_height).sum::<f64>()
            + 12.
            + if q.category.is_empty() {
                0.
            } else {
                wrapped_lines(
                    &format!("Categoria: {}", q.category.replace('/', " / ")),
                    8.,
                    false,
                )
                .len() as f64
                    * 9.76
            };
        // Keep ordinary questions whole; oversized questions flow across pages.
        let fresh_capacity = HEIGHT - 2. * MARGIN - 70.;
        p.ensure(if height <= fresh_capacity {
            height
        } else {
            45.
        });
        if !q.category.is_empty() {
            p.line(
                &format!("Categoria: {}", q.category.replace('/', " / ")),
                8.,
                false,
            );
        }
        p.line(&title, 10.5, true);
        p.gap(3.);
        for pic in &q.pictures {
            p.picture(pic)
        }
        for (j, a) in q.answers.iter().enumerate() {
            p.line(
                &format!("   {}) {}", (b'a' + j as u8) as char, a.text),
                10.5,
                false,
            );
            for pic in &a.pictures {
                p.picture(pic)
            }
        }
        p.gap(9.);
    }
    p
}
fn picture_height(pic: &Picture) -> f64 {
    image::load_from_memory(&pic.data)
        .map(|img| {
            let rgb = img.thumbnail(1280, 1280);
            let w = rgb.width() as f64;
            let h = rgb.height() as f64;
            h * (350. / w).min(220. / h).min(1.) + 12.
        })
        .unwrap_or(0.)
}
fn key(batch: &Batch, s: &StudentExam) -> Pdf {
    let mut p = Pdf::new("Gabarito individual", &s.name);
    p.line(&batch.title, 12., true);
    p.line(
        &format!("Identificador do lote: {}", batch.seed),
        10.,
        false,
    );
    p.gap(16.);
    for (i, q) in s.questions.iter().enumerate() {
        p.ensure(65.);
        if !q.category.is_empty() {
            p.line(
                &format!("Categoria: {}", q.category.replace('/', " / ")),
                8.,
                false,
            );
        }
        p.line(
            &format!("Questão {} - {}: {}", i + 1, q.title, letters(q)),
            11.,
            true,
        );
        for a in q.answers.iter().filter(|a| a.correct) {
            p.line(&a.text, 10., false)
        }
        p.line(
            &q.values
                .iter()
                .map(|(k, v)| format!("{k} = {}", fmt(*v)))
                .collect::<Vec<_>>()
                .join("  |  "),
            9.,
            false,
        );
        p.gap(10.);
    }
    p
}
fn safe(name: &str) -> String {
    // MoodTest.py App._sanitizar_nome, source offset 0x1340f, called at 0x13be2:
    // trim, collapse invalid-character runs, then collapse whitespace runs.
    // These are separate passes: "Ana / Silva" becomes "Ana___Silva".
    let mut normalized = String::new();
    let mut previous_invalid = false;
    for c in name.trim().chars() {
        let invalid = matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|');
        if !invalid || !previous_invalid {
            normalized.push(if invalid { '_' } else { c });
        }
        previous_invalid = invalid;
    }
    let mut s = String::new();
    let mut previous_space = false;
    for c in normalized.chars() {
        let space = c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}');
        if !space || !previous_space {
            s.push(if space { '_' } else { c });
        }
        previous_space = space;
    }
    // Product limit retained; the original does not truncate names.
    let s: String = s.chars().take(90).collect();
    if s.is_empty() {
        "aluno".into()
    } else {
        s
    }
}
pub fn export(batch: &Batch, base: &Path) -> Result<PathBuf, String> {
    if batch.students.is_empty() {
        return Err("O lote está vazio.".into());
    }
    fs::create_dir_all(base).map_err(|e| e.to_string())?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let folder = base.join(format!("provas_{stamp}"));
    fs::create_dir(&folder).map_err(|e| e.to_string())?;
    let result = (|| -> Result<(), String> {
        let mut all: Option<Pdf> = None;
        let mut keys = Pdf::new("Gabarito consolidado", &batch.title);
        keys.line(&format!("Identificador: {}", batch.seed), 10., false);
        keys.gap(14.);
        let mut csv = String::from("\u{feff}Aluno;Questão;Gabarito;Resposta correta;Categoria\n");
        for (i, s) in batch.students.iter().enumerate() {
            let prefix = format!("{:03}_{}", i + 1, safe(&s.name));
            fs::write(
                folder.join(format!("moodle_{prefix}.xml")),
                crate::moodle::instances_xml(&s.questions, &[])?,
            )
            .map_err(|e| e.to_string())?;
            fs::write(
                folder.join(format!("prova_{prefix}.pdf")),
                exam(batch, s).bytes(),
            )
            .map_err(|e| e.to_string())?;
            fs::write(
                folder.join(format!("gabarito_{prefix}.pdf")),
                key(batch, s).bytes(),
            )
            .map_err(|e| e.to_string())?;
            if let Some(a) = &mut all {
                a.append(exam(batch, s))
            } else {
                all = Some(exam(batch, s))
            }
            keys.ensure(60.);
            keys.line(&s.name, 11., true);
            keys.line(
                &s.questions
                    .iter()
                    .enumerate()
                    .map(|(j, q)| format!("Q{}: {}", j + 1, letters(q)))
                    .collect::<Vec<_>>()
                    .join("   |   "),
                10.,
                false,
            );
            keys.gap(12.);
            for (j, q) in s.questions.iter().enumerate() {
                let quote = |s: &str| format!("\"{}\"", s.replace('"', "\"\""));
                csv.push_str(&format!(
                    "{};{};{};{};{}\n",
                    quote(&s.name),
                    j + 1,
                    letters(q),
                    quote(
                        &q.answers
                            .iter()
                            .filter(|a| a.correct)
                            .map(|a| a.text.clone())
                            .collect::<Vec<_>>()
                            .join(" / ")
                    ),
                    quote(&q.category)
                ));
            }
        }
        fs::write(folder.join("todas_as_provas.pdf"), all.unwrap().bytes())
            .map_err(|e| e.to_string())?;
        fs::write(folder.join("gabarito_consolidado.pdf"), keys.bytes())
            .map_err(|e| e.to_string())?;
        fs::write(folder.join("gabaritos.csv"), csv).map_err(|e| e.to_string())?;
        fs::write(
            folder.join("lote.json"),
            serde_json::to_vec_pretty(batch).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    })();
    if let Err(e) = result {
        return Err(format!(
            "Falha ao concluir o lote: {e}. A pasta {} pode conter arquivos parciais.",
            folder.display()
        ));
    }
    Ok(folder)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exam_logo_categories_and_legacy_batches_roundtrip() {
        let bank = examples();
        let mut batch = generate(
            &bank,
            &[("Mecânica".into(), 1)],
            "Aluno Exemplo\nOutra Aluna",
            2026,
            "Avaliação de Física",
            "Colégio Exemplo",
            "Maria Silva",
            "10/11/2026",
            "3º A",
        )
        .unwrap();
        let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("checks/header-design");
        fs::create_dir_all(&folder).unwrap();
        fs::write(
            folder.join("without-logo.pdf"),
            exam(&batch, &batch.students[0]).bytes(),
        )
        .unwrap();
        let mut legacy = serde_json::to_value(&batch).unwrap();
        legacy.as_object_mut().unwrap().remove("logo");
        for student in legacy["students"].as_array_mut().unwrap() {
            for question in student["questions"].as_array_mut().unwrap() {
                question.as_object_mut().unwrap().remove("category");
            }
        }
        let old: Batch = serde_json::from_value(legacy).unwrap();
        assert!(old.logo.is_none());
        assert!(old.students[0].questions[0].category.is_empty());
        batch.logo = Some(Picture {
            name: "Logo de exemplo".into(),
            data: include_bytes!("../assets/test-craft.png").to_vec(),
        });
        let reopened: Batch = serde_json::from_slice(&serde_json::to_vec(&batch).unwrap()).unwrap();
        assert_eq!(reopened.logo, batch.logo);
        let pdf = exam(&reopened, &reopened.students[0]);
        assert_eq!(pdf.pictures.len(), 1);
        assert!(pdf.page.contains("Categoria:"));
        fs::write(folder.join("with-logo.pdf"), pdf.bytes()).unwrap();
        let output = export(&reopened, &folder).unwrap();
        let csv = fs::read_to_string(output.join("gabaritos.csv")).unwrap();
        assert!(csv.lines().next().unwrap().ends_with(";Categoria"));
        assert!(csv.contains("Mecânica"));
        batch.title = "Avaliação com um título longo de Física e Ciências da Natureza ".repeat(4);
        batch.institution = "Instituição com nome extenso ".repeat(4);
        fs::write(
            folder.join("long-header-logo.pdf"),
            exam(&batch, &batch.students[0]).bytes(),
        )
        .unwrap();
    }
    #[test]
    fn compact_exams_keep_questions_together_and_wrap_long_headers() {
        let bank = examples();
        let mut batch = generate(
            &bank,
            &[("Mecânica".into(), 1)],
            "Aluno Exemplo",
            2026,
            "Avaliação de Física",
            "Escola Exemplo",
            "Maria Silva",
            "10/11/2026",
            "3º A",
        )
        .unwrap();
        let mut q = batch.students[0].questions[0].clone();
        q.text = "Qual alternativa está correta?".into();
        q.pictures.clear();
        q.answers = (0..4)
            .map(|i| Answer {
                text: format!("Alternativa {}", i + 1),
                correct: i == 0,
                pictures: vec![],
            })
            .collect();
        batch.students[0].questions = vec![q; 20];
        let pages = exam(&batch, &batch.students[0]).finish_pages();
        assert!(
            pages.len() <= 3,
            "20 short questions should use at most three pages"
        );
        for page in &pages {
            assert_eq!(
                page.matches("correta?").count(),
                page.matches("Alternativa 4").count(),
                "Question separated from its answers"
            );
        }
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("checks/compact-pdf");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("exam.pdf"),
            exam(&batch, &batch.students[0]).bytes(),
        )
        .unwrap();
        batch.title =
            "Avaliação com título muito extenso de Física e Ciências da Natureza ".repeat(4);
        batch.students[0].name = "Nome de estudante composto ".repeat(6);
        batch.institution = "Instituição de ensino com nome extenso ".repeat(4);
        fs::write(
            dir.join("long-header.pdf"),
            exam(&batch, &batch.students[0]).bytes(),
        )
        .unwrap();
        for line in wrapped_lines(&batch.title, 16., true) {
            assert!(text_width(&line, 16., true) <= WIDTH - 2. * MARGIN);
        }
    }
    #[test]
    fn student_filenames_match_original() {
        let mechanics: serde_json::Value =
            serde_json::from_str(include_str!("../mechanics.json")).unwrap();
        for case in mechanics["original"]["filename_cases"].as_array().unwrap() {
            let name = case["name"].as_str().unwrap();
            // Keep this app's existing 90-character filename limit.
            let expected: String = case["expected"]
                .as_str()
                .unwrap()
                .chars()
                .take(90)
                .collect();
            assert_eq!(safe(name), expected, "original filename for {name:?}");
        }
    }
    #[test]
    fn blank_answer_tables_match_original() {
        let mechanics: serde_json::Value =
            serde_json::from_str(include_str!("../mechanics.json")).unwrap();
        let bank = examples();
        let mut batch = generate(
            &bank,
            &[("Mecânica".into(), 1)],
            "Aluno Teste",
            2026,
            "Prova de referência",
            "Escola",
            "Professor",
            "09/11/2026",
            "A",
        )
        .unwrap();
        let question = batch.students[0].questions[0].clone();
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("checks/answer-tables");
        fs::create_dir_all(&dir).unwrap();
        for case in mechanics["original"]["blank_answer_table_cases"]
            .as_array()
            .unwrap()
        {
            let count = case["question_count"].as_u64().unwrap() as usize;
            batch.students[0].questions = vec![question.clone(); count];
            let bytes = exam(&batch, &batch.students[0]).bytes();
            let content = String::from_utf8_lossy(&bytes);
            for block in case["headers"].as_array().unwrap() {
                for label in block.as_array().unwrap() {
                    let marker = format!("({}) Tj", label.as_str().unwrap());
                    assert_eq!(
                        content.matches(&marker).count(),
                        1,
                        "missing or repeated {marker} for {count} questions"
                    );
                }
            }
            fs::write(dir.join(format!("exam-{count}.pdf")), bytes).unwrap();
        }
        let mut crossing = Pdf::new("Tabela com quebra de página", "Aluno");
        crossing.y = MARGIN + 110.;
        crossing.blank_answer_tables(31);
        assert_eq!(crossing.pages.len(), 1);
        assert!(crossing.pages[0].contains("(Q15) Tj"));
        assert!(!crossing.pages[0].contains("(Q16) Tj"));
        assert!(crossing.page.contains("(Q16) Tj"));
        assert!(crossing.page.contains("(Q31) Tj"));
    }
    #[test]
    fn answer_letters_match_original() {
        let mechanics: serde_json::Value =
            serde_json::from_str(include_str!("../mechanics.json")).unwrap();
        for case in mechanics["original"]["answer_letter_cases"]
            .as_array()
            .unwrap()
        {
            let indices: Vec<usize> = serde_json::from_value(case["indices"].clone()).unwrap();
            let q = Instance {
                title: String::new(),
                category: String::new(),
                text: String::new(),
                values: Default::default(),
                pictures: vec![],
                answers: (0..8)
                    .map(|i| Answer {
                        text: i.to_string(),
                        correct: indices.contains(&i),
                        pictures: vec![],
                    })
                    .collect(),
            };
            assert_eq!(letters(&q), case["expected"].as_str().unwrap());
        }
    }
    #[test]
    fn pictures_symbols_and_combined_pdf() {
        let mut bytes = std::io::Cursor::new(Vec::new());
        let img = image::RgbImage::from_fn(120, 50, |x, y| {
            if x % 20 < 3 || y % 20 < 3 {
                image::Rgb([35, 110, 95])
            } else {
                image::Rgb([240, 247, 244])
            }
        });
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        let picture = Picture {
            name: "Diagrama de teste".into(),
            data: bytes.into_inner(),
        };
        check_picture(&picture).unwrap();
        let mut bank = examples();
        bank.questions[3].pictures.push(picture.clone());
        bank.questions[3].answers[0].pictures.push(picture);
        bank.questions[3].text =
            "Um resistor de {r} Ω está sob tensão de {u} V. Qual é a corrente?".into();
        let batch = generate(
            &bank,
            &[("Eletricidade".into(), 2)],
            "Ágata
João",
            2026,
            "Teste de imagens Ω",
            "",
            "",
            "",
            "",
        )
        .unwrap();
        let folder = export(
            &batch,
            &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("checks/image-output"),
        )
        .unwrap();
        let data = std::fs::read(folder.join("todas_as_provas.pdf")).unwrap();
        let text = String::from_utf8_lossy(&data);
        assert!(text.contains("/Subtype /Image"));
        assert!(text.contains("/BaseFont /Symbol"));
        // The answer tables can add pages. The combined PDF must preserve all
        // pages of both individual exams, including image resources.
        let count_pages = |data: &[u8]| {
            String::from_utf8_lossy(data)
                .matches("/Type /Page ")
                .count()
        };
        let individual_pages: usize = fs::read_dir(&folder)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("prova_")
            })
            .map(|path| count_pages(&fs::read(path).unwrap()))
            .sum();
        assert!(individual_pages >= 2);
        assert_eq!(count_pages(&data), individual_pages);
    }
    #[test]
    fn pdf_has_valid_xref_and_accents() {
        let mut p = Pdf::new("Avaliação", "João");
        for i in 0..100 {
            p.line(&format!("Questão {i}: ação e eletricidade"), 11., false)
        }
        let b = p.bytes();
        let s = String::from_utf8_lossy(&b);
        assert!(s.starts_with("%PDF-1.4"));
        assert!(s.contains("\\343"));
        assert!(s.contains("/Count 2"));
        let start = s
            .rsplit("startxref\n")
            .nth(0)
            .unwrap()
            .split('\n')
            .next()
            .unwrap()
            .parse::<usize>()
            .unwrap();
        assert_eq!(&b[start..start + 4], b"xref");
    }
}
