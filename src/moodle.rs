//! Moodle XML exports use fixed multichoice versions, matching the generated PDFs.
//! Format: https://docs.moodle.org/502/en/Moodle_XML_format
use crate::model::*;
use std::collections::BTreeSet;

fn escape(text: &str) -> String {
    let mut output = String::new();
    for c in text.chars() {
        match c {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&apos;"),
            c if c < ' ' && !matches!(c, '\n' | '\r' | '\t') => output.push('\u{fffd}'),
            _ => output.push(c),
        }
    }
    output
}

fn base64(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(data.len().div_ceil(3) * 4);
    for bytes in data.chunks(3) {
        let n = ((bytes[0] as u32) << 16)
            | ((bytes.get(1).copied().unwrap_or(0) as u32) << 8)
            | bytes.get(2).copied().unwrap_or(0) as u32;
        output.push(ALPHABET[(n >> 18) as usize] as char);
        output.push(ALPHABET[((n >> 12) & 63) as usize] as char);
        output.push(if bytes.len() > 1 {
            ALPHABET[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        output.push(if bytes.len() > 2 {
            ALPHABET[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    output
}

fn content(text: &str, pictures: &[Picture], prefix: &str) -> Result<String, String> {
    let mut html = format!("<p>{}</p>", escape(text).replace('\n', "<br />"));
    let mut files = String::new();
    for (i, picture) in pictures.iter().enumerate() {
        check_picture(picture)?;
        let extension = match image::guess_format(&picture.data).map_err(|e| e.to_string())? {
            image::ImageFormat::Png => "png",
            image::ImageFormat::Jpeg => "jpg",
            _ => return Err("O Moodle XML aceita imagens PNG ou JPEG.".into()),
        };
        let name = format!("{prefix}_{i}.{extension}");
        html.push_str(&format!(
            "<p><img src=\"@@PLUGINFILE@@/{name}\" alt=\"{}\" /></p>",
            escape(&picture.name)
        ));
        files.push_str(&format!(
            "<file name=\"{name}\" path=\"/\" encoding=\"base64\">{}</file>\n",
            base64(&picture.data)
        ));
    }
    Ok(format!("<text>{}</text>\n{files}", escape(&html)))
}

fn category(output: &mut String, path: &str) {
    output.push_str(&format!(
        "<question type=\"category\"><category><text>$course$/{}</text></category></question>\n",
        escape(path)
    ));
}

pub fn instances_xml(questions: &[Instance], labels: &[String]) -> Result<String, String> {
    if questions.is_empty() {
        return Err("Não há questões prontas e válidas para exportar ao Moodle.".into());
    }
    let mut output = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<quiz>\n");
    // Preserve empty categories as well as the category of each question.
    for label in labels.iter().collect::<BTreeSet<_>>() {
        category(&mut output, label);
    }
    for q in questions {
        let correct = q.answers.iter().filter(|a| a.correct).count();
        if correct == 0 || q.answers.len() < 2 || q.answers.len() > 8 {
            return Err(format!(
                "A questão '{}' precisa de alternativas e gabarito válidos.",
                q.title
            ));
        }
        category(
            &mut output,
            if q.category.is_empty() {
                "Test Craft"
            } else {
                &q.category
            },
        );
        output.push_str(&format!("<question type=\"multichoice\">\n<name><text>{}</text></name>\n<questiontext format=\"html\">{}</questiontext>\n<defaultgrade>1</defaultgrade><penalty>0.3333333</penalty><hidden>0</hidden>\n<single>{}</single><shuffleanswers>0</shuffleanswers><answernumbering>abc</answernumbering>\n", escape(&q.title), content(&q.text, &q.pictures, "question")?, if correct == 1 {"true"} else {"false"}));
        for (i, answer) in q.answers.iter().enumerate() {
            let fraction = if answer.correct {
                100. / correct as f64
            } else if correct > 1 {
                -100.
            } else {
                0.
            };
            output.push_str(&format!(
                "<answer fraction=\"{fraction:.7}\" format=\"html\">{}</answer>\n",
                content(&answer.text, &answer.pictures, &format!("answer_{i}"))?
            ));
        }
        output.push_str("</question>\n");
    }
    output.push_str("</quiz>\n");
    Ok(output)
}

pub fn bank_xml(bank: &Bank, seed: u64) -> Result<(String, usize), String> {
    let questions = bank
        .questions
        .iter()
        .filter(|q| eligible(q))
        .map(|q| preview(q, seed))
        .collect::<Result<Vec<_>, _>>()?;
    let count = questions.len();
    Ok((instances_xml(&questions, &categories(bank))?, count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use xml::reader::{EventReader, XmlEvent};

    fn parse(xml: &str) -> Vec<XmlEvent> {
        EventReader::from_str(xml)
            .into_iter()
            .map(|e| e.expect("well-formed Moodle XML"))
            .collect()
    }

    #[test]
    fn bank_export_roundtrips_categories_images_grades_and_fixed_values() {
        let mut bank = examples();
        bank.questions.truncate(1);
        bank.questions[0].title = "Óptica < & > \" ]]>".into();
        bank.questions[0].category = "Ciências/Física & Óptica".into();
        bank.category_labels = vec!["Vazia/Subcategoria".into()];
        let picture = Picture {
            name: "Figura < & \"".into(),
            data: include_bytes!("../assets/test-craft.png").to_vec(),
        };
        bank.questions[0].pictures = vec![picture.clone()];
        bank.questions[0].answers[0].pictures = vec![picture.clone()];
        let mut draft = bank.questions[0].clone();
        draft.id += 1;
        draft.status = QuestionStatus::Draft;
        bank.questions.push(draft);
        let mut broken = bank.questions[0].clone();
        broken.id += 2;
        broken.text.clear();
        bank.questions.push(broken);
        let expected = preview(&bank.questions[0], 2026).unwrap();
        let (xml, count) = bank_xml(&bank, 2026).unwrap();
        assert_eq!(count, 1);
        parse(&xml);
        assert!(xml.contains("<single>true</single>"));
        assert!(xml.contains("<shuffleanswers>0</shuffleanswers>"));
        assert_eq!(xml.matches("fraction=\"100.0000000\"").count(), 1);
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("checks/moodle-export");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("roundtrip.xml");
        std::fs::write(&path, xml).unwrap();
        let reopened = import_xml(&path).unwrap();
        assert_eq!(reopened.questions.len(), 1);
        let actual = &reopened.questions[0];
        assert_eq!(actual.title, expected.title);
        assert_eq!(actual.category, expected.category);
        assert_eq!(actual.text.trim(), expected.text.trim());
        assert!(actual.variables.is_empty());
        assert!(
            actual.import_issues.is_empty(),
            "{:?}",
            actual.import_issues
        );
        assert!(categories(&reopened).contains(&"Vazia/Subcategoria".into()));
        assert_eq!(actual.pictures[0].data, picture.data);
        assert_eq!(actual.answers[0].pictures[0].data, picture.data);
        for (a, b) in actual.answers.iter().zip(&expected.answers) {
            assert_eq!(a.text.trim(), b.text.trim());
            assert_eq!(a.correct, b.correct);
        }
    }

    #[test]
    fn multiple_response_scoring_and_empty_exports() {
        let bank = examples();
        let mut q = preview(&bank.questions[0], 42).unwrap();
        for (i, a) in q.answers.iter_mut().enumerate() {
            a.correct = i < 2;
        }
        let xml = instances_xml(&[q.clone()], &[]).unwrap();
        parse(&xml);
        assert!(xml.contains("<single>false</single>"));
        assert_eq!(xml.matches("fraction=\"50.0000000\"").count(), 2);
        assert!(xml.contains("fraction=\"-100.0000000\""));
        assert!(instances_xml(&[], &[]).is_err());
        for a in &mut q.answers {
            a.correct = false;
        }
        assert!(instances_xml(&[q], &[]).is_err());
        let mut bank = bank;
        for q in &mut bank.questions {
            q.status = QuestionStatus::Draft;
        }
        assert!(bank_xml(&bank, 42).is_err());
    }

    #[test]
    fn batch_xml_matches_each_students_rendered_questions_and_answer_order() {
        let batch = generate(
            &examples(),
            &[("Mecânica".into(), 2)],
            "Aluno A\nAluno B",
            421,
            "Prova",
            "Escola",
            "Professor",
            "11/11/2026",
            "Turma",
        )
        .unwrap();
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("checks/moodle-batch");
        let output = crate::pdf::export(&batch, &dir).unwrap();
        for (i, student) in batch.students.iter().enumerate() {
            let path = output.join(format!(
                "moodle_{:03}_{}.xml",
                i + 1,
                student.name.replace(' ', "_")
            ));
            let xml = std::fs::read_to_string(&path).unwrap();
            parse(&xml);
            let imported = import_xml(&path).unwrap();
            assert_eq!(imported.questions.len(), student.questions.len());
            for (actual, expected) in imported.questions.iter().zip(&student.questions) {
                assert_eq!(actual.text.trim(), expected.text.trim());
                assert_eq!(actual.category, expected.category);
                assert_eq!(actual.answers.len(), expected.answers.len());
                for (a, b) in actual.answers.iter().zip(&expected.answers) {
                    assert_eq!(a.text.trim(), b.text.trim());
                    assert_eq!(a.correct, b.correct);
                }
            }
        }
    }
}
