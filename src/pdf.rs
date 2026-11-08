use crate::model::*;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
pub const WIDTH: f64 = 595.2756;
pub const HEIGHT: f64 = 841.8898;
pub const MARGIN: f64 = 56.6929;

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
        self.page.push_str("0.08 0.28 0.25 rg\n");
        self.draw(&self.title.clone(), 14., true);
        self.page.push_str("0.22 0.26 0.30 rg\n");
        self.line(
            &format!("{}  |  Página {}", self.name, self.number),
            10.,
            false,
        );
        self.y -= 10.;
        self.page.push_str(&format!(
            "0.82 0.86 0.86 RG {MARGIN} {} m {} {} l S\n",
            self.y,
            WIDTH - MARGIN,
            self.y
        ));
        self.y -= 20.;
    }
    fn draw(&mut self, t: &str, size: f64, bold: bool) {
        self.page.push_str(&format!(
            "BT 1 0 0 1 {MARGIN:.3} {:.3} Tm {} ET\n",
            self.y,
            text_commands(t, size, bold)
        ));
        self.y -= size * 1.45;
    }
    fn next(&mut self) {
        self.pages.push(std::mem::take(&mut self.page));
        self.begin();
    }
    pub fn line(&mut self, t: &str, size: f64, bold: bool) {
        let max = WIDTH - 2. * MARGIN;
        let mut line = String::new();
        for para in t.split('\n') {
            for word in para.split_whitespace() {
                if text_width(&line, size, bold) + text_width(word, size, bold) > max
                    && !line.is_empty()
                {
                    if self.y < MARGIN + 30. {
                        self.next()
                    }
                    self.draw(&line, size, bold);
                    line.clear();
                }
                // Split a single oversized token instead of clipping it.
                for c in word.chars() {
                    if text_width(&line, size, bold) + text_width(&c.to_string(), size, bold) > max
                    {
                        if self.y < MARGIN + 30. {
                            self.next()
                        }
                        self.draw(&line, size, bold);
                        line.clear();
                    }
                    line.push(c);
                }
                line.push(' ');
            }
            if self.y < MARGIN + 30. {
                self.next()
            }
            self.draw(line.trim_end(), size, bold);
            line.clear();
        }
    }
    pub fn gap(&mut self, n: f64) {
        self.y -= n;
    }
    pub fn ensure(&mut self, height: f64) {
        if self.y - height < MARGIN + 20. {
            self.next()
        }
    }
    pub fn picture(&mut self, pic: &Picture) {
        use std::io::Write;
        if let Ok(img) = image::load_from_memory(&pic.data) {
            let rgb = img.thumbnail(1280, 1280).to_rgb8();
            let (w, h) = rgb.dimensions();
            let scale = (350. / w as f64).min(220. / h as f64).min(1.);
            let dw = w as f64 * scale;
            let dh = h as f64 * scale;
            self.ensure(dh + 16.);
            let id = self.pictures.len();
            let mut encoder =
                flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            if encoder.write_all(rgb.as_raw()).is_ok() {
                if let Ok(data) = encoder.finish() {
                    self.page.push_str(&format!(
                        "q {dw} 0 0 {dh} {MARGIN} {} cm /Im{id} Do Q\n",
                        self.y - dh
                    ));
                    self.y -= dh + 12.;
                    self.pictures.push(PdfImage {
                        width: w,
                        height: h,
                        data,
                    });
                }
            }
        }
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
    q.answers
        .iter()
        .enumerate()
        .filter(|(_, a)| a.correct)
        .map(|(i, _)| ((b'A' + i as u8) as char).to_string())
        .collect::<Vec<_>>()
        .join("/")
}
fn exam(batch: &Batch, s: &StudentExam) -> Pdf {
    let mut p = Pdf::new(&batch.title, &s.name);
    for t in [
        &batch.institution,
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
    p.gap(12.);
    p.line(
        "Responda às questões abaixo. Assinale as alternativas corretas.",
        10.,
        false,
    );
    p.gap(10.);
    p.line(
        &format!(
            "Respostas: {}",
            (1..=s.questions.len())
                .map(|i| format!("{i}: ____"))
                .collect::<Vec<_>>()
                .join("   ")
        ),
        10.,
        false,
    );
    p.gap(16.);
    for (i, q) in s.questions.iter().enumerate() {
        p.ensure(110.);
        p.line(&format!("{}. {}", i + 1, q.text), 11., true);
        p.gap(5.);
        for pic in &q.pictures {
            p.picture(pic)
        }
        for (j, a) in q.answers.iter().enumerate() {
            p.line(
                &format!("   {}) {}", (b'A' + j as u8) as char, a.text),
                11.,
                false,
            );
            for pic in &a.pictures {
                p.picture(pic)
            }
        }
        p.gap(18.);
    }
    p
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
    let s: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(90)
        .collect();
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
        let mut csv = String::from("\u{feff}Aluno;Questão;Gabarito;Resposta correta\n");
        for (i, s) in batch.students.iter().enumerate() {
            let prefix = format!("{:03}_{}", i + 1, safe(&s.name));
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
                    "{};{};{};{}\n",
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
                    )
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
        assert!(text.contains("/Count 2"));
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
        assert!(s.contains("/Count 3") || s.contains("/Count 4"));
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
