pub struct StdRng(u64);
impl StdRng {
    pub fn seed_from_u64(seed: u64) -> Self {
        Self(seed)
    }
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    pub fn gen_range(&mut self, r: std::ops::Range<usize>) -> usize {
        let n = (r.end - r.start) as u64;
        let limit = u64::MAX - u64::MAX % n;
        loop {
            let x = self.next();
            if x < limit {
                return r.start + (x % n) as usize;
            }
        }
    }
}
trait Shuffle {
    fn shuffle(&mut self, rng: &mut StdRng);
}
impl<T> Shuffle for [T] {
    fn shuffle(&mut self, rng: &mut StdRng) {
        for i in (1..self.len()).rev() {
            let j = rng.gen_range(0..i + 1);
            self.swap(i, j)
        }
    }
}
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub const DECIMALS: usize = 2;
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Variable {
    pub name: String,
    pub min: f64,
    pub max: f64,
    pub decimals: u32,
    #[serde(default)]
    pub values: Vec<f64>,
}
impl Variable {
    pub fn count(&self) -> usize {
        if !self.values.is_empty() {
            self.values.len()
        } else {
            (((self.max - self.min) * 10f64.powi(self.decimals.min(6) as i32)).round() as usize)
                .saturating_add(1)
        }
    }
    pub fn at(&self, i: usize) -> f64 {
        if !self.values.is_empty() {
            self.values[i % self.values.len()]
        } else {
            self.min + (i % self.count()) as f64 / 10f64.powi(self.decimals as i32)
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Picture {
    pub name: String,
    pub data: Vec<u8>,
}
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Answer {
    pub text: String,
    pub correct: bool,
    #[serde(default)]
    pub pictures: Vec<Picture>,
}
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Question {
    pub id: u64,
    pub title: String,
    pub category: String,
    pub text: String,
    #[serde(default)]
    pub pictures: Vec<Picture>,
    pub variables: Vec<Variable>,
    pub answers: Vec<Answer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub import_issues: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_xml: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Bank {
    pub version: u32,
    pub questions: Vec<Question>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub category_labels: Vec<String>,
}
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Instance {
    pub title: String,
    pub text: String,
    pub values: BTreeMap<String, f64>,
    #[serde(default)]
    pub pictures: Vec<Picture>,
    pub answers: Vec<Answer>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct StudentExam {
    pub name: String,
    pub questions: Vec<Instance>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Batch {
    pub version: u32,
    pub seed: u64,
    #[serde(default)]
    pub plan: Vec<(String, usize)>,
    pub title: String,
    pub institution: String,
    pub teacher: String,
    pub date: String,
    pub class_name: String,
    pub students: Vec<StudentExam>,
}

pub fn fmt(x: f64) -> String {
    if !x.is_finite() || x.abs() > 1e15 {
        return x.to_string();
    }
    // Round the shortest decimal representation, as Decimal(str(x)) does in the original.
    let raw = x.abs().to_string();
    let (mantissa, exp) = raw
        .split_once(['e', 'E'])
        .map(|(a, b)| (a, b.parse::<i32>().unwrap_or(0)))
        .unwrap_or((&raw, 0));
    let point = mantissa.find('.').unwrap_or(mantissa.len()) as i32 + exp;
    let digits = mantissa.replace('.', "");
    let target = point + DECIMALS as i32;
    let scaled: u64 = if target < 0 {
        0
    } else if target as usize >= digits.len() {
        digits
            .parse::<u64>()
            .unwrap_or(0)
            .saturating_mul(10u64.pow((target as usize - digits.len()) as u32))
    } else {
        let n = digits[..target as usize].parse::<u64>().unwrap_or(0);
        n + u64::from(digits.as_bytes()[target as usize] >= b'5')
    };
    let scale = 10u64.pow(DECIMALS as u32);
    let whole = (scaled / scale).to_string();
    let fraction = scaled % scale;
    let mut out = String::new();
    if x < 0. && scaled > 0 {
        out.push('-')
    }
    for (i, c) in whole.chars().enumerate() {
        if i > 0 && (whole.len() - i) % 3 == 0 {
            out.push('.')
        }
        out.push(c)
    }
    if fraction != 0 {
        out.push_str(&format!(",{fraction:0width$}", width = DECIMALS))
    }
    out
}

// A small numeric grammar: never executes user code.
struct Parser<'a> {
    chars: Vec<char>,
    pos: usize,
    vars: &'a BTreeMap<String, f64>,
    depth: usize,
}
impl<'a> Parser<'a> {
    fn peek(&self) -> char {
        self.chars.get(self.pos).copied().unwrap_or('\0')
    }
    fn eat(&mut self, c: char) -> bool {
        if self.peek() == c {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn expr(&mut self) -> Result<f64, String> {
        self.depth += 1;
        if self.depth > 64 {
            return Err("Fórmula muito complexa.".into());
        }
        let mut x = self.term()?;
        loop {
            if self.eat('+') {
                x += self.term()?
            } else if self.eat('-') {
                x -= self.term()?
            } else {
                break;
            }
        }
        self.depth -= 1;
        Ok(x)
    }
    fn term(&mut self) -> Result<f64, String> {
        let mut x = self.unary()?;
        loop {
            if self.peek() == '*' && self.chars.get(self.pos + 1) != Some(&'*') {
                self.pos += 1;
                x *= self.unary()?
            } else if self.eat('/') {
                let d = self.unary()?;
                if d == 0.0 {
                    return Err("Divisão por zero.".into());
                }
                x /= d;
            } else {
                break;
            }
        }
        Ok(x)
    }
    fn unary(&mut self) -> Result<f64, String> {
        if self.eat('-') {
            return Ok(-self.unary()?);
        }
        if self.eat('+') {
            return self.unary();
        }
        self.power()
    }
    fn power(&mut self) -> Result<f64, String> {
        let x = self.atom()?;
        if self.eat('^') {
            return self.exponent(x);
        }
        if self.peek() == '*' && self.chars.get(self.pos + 1) == Some(&'*') {
            self.pos += 2;
            return self.exponent(x);
        }
        Ok(x)
    }
    fn exponent(&mut self, base: f64) -> Result<f64, String> {
        let exponent = self.unary()?;
        // MoodTest.py _eval_ast (0xc7b), operator call at source offset 0xea4:
        // Python raises at zero to a negative power, before an outer ^0 can hide it.
        if base == 0.0 && exponent < 0.0 {
            return Err("Zero não pode ser elevado a um expoente negativo.".into());
        }
        Ok(base.powf(exponent))
    }
    fn atom(&mut self) -> Result<f64, String> {
        if self.eat('(') {
            let x = self.expr()?;
            if !self.eat(')') {
                return Err("Feche os parênteses da fórmula.".into());
            }
            return Ok(x);
        }
        if self.eat('{') {
            let start = self.pos;
            while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
                self.pos += 1
            }
            let name: String = self.chars[start..self.pos].iter().collect();
            if !self.eat('}') {
                return Err("Feche a variável com }.".into());
            }
            return self
                .vars
                .get(&name)
                .copied()
                .ok_or(format!("Variável {{{name}}} não cadastrada."));
        }
        if self.peek().is_ascii_digit() || self.peek() == '.' {
            let start = self.pos;
            while self.peek().is_ascii_digit() || self.peek() == '.' {
                self.pos += 1
            }
            if matches!(self.peek(), 'e' | 'E') {
                self.pos += 1;
                if matches!(self.peek(), '+' | '-') {
                    self.pos += 1
                }
                while self.peek().is_ascii_digit() {
                    self.pos += 1
                }
            }
            return self.chars[start..self.pos]
                .iter()
                .collect::<String>()
                .parse()
                .map_err(|_| "Número inválido.".into());
        }
        if self.peek().is_ascii_alphabetic() || self.peek() == '_' {
            let start = self.pos;
            while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
                self.pos += 1
            }
            let name: String = self.chars[start..self.pos].iter().collect();
            if self.eat('(') {
                let x = self.expr()?;
                if !self.eat(')') {
                    return Err("Feche os parênteses da função.".into());
                }
                let value = match name.as_str() {
                    "sqrt" => Ok(x.sqrt()),
                    "abs" => Ok(x.abs()),
                    "sin" => Ok(x.sin()),
                    "cos" => Ok(x.cos()),
                    "tan" => Ok(x.tan()),
                    "log" | "ln" => Ok(x.ln()),
                    "log10" => Ok(x.log10()),
                    "exp" => Ok(x.exp()),
                    _ => Err(format!("Função {name} não permitida.")),
                }?;
                // _eval_ast's fn(*args), source offset 0x1112, propagates math
                // domain/overflow errors immediately, even inside another operation.
                if !value.is_finite() {
                    return Err(format!(
                        "Resultado indefinido ou fora do limite numérico em {name}."
                    ));
                }
                return Ok(value);
            }
            // MoodTest.py _eval_ast (0xc7b), context lookup at 0x118c:
            // dataset values precede constants; calls above still use allowed functions.
            if let Some(value) = self.vars.get(&name) {
                return Ok(*value);
            }
            return match name.as_str() {
                "pi" => Ok(std::f64::consts::PI),
                "e" => Ok(std::f64::consts::E),
                _ => self
                    .vars
                    .get(&name)
                    .copied()
                    .ok_or(format!("Variável {name} não cadastrada.")),
            };
        }
        Err("Fórmula incompleta ou símbolo não permitido.".into())
    }
}
pub fn eval(expr: &str, vars: &BTreeMap<String, f64>) -> Result<f64, String> {
    if expr.len() > 4096 {
        return Err("A fórmula deve ter até 4.096 caracteres.".into());
    }
    // MoodTest.py:219 substitutes braced values as decimal text before parsing.
    // In particular, {x}^2 with x=-2 means -2^2; ({x})^2 preserves the grouping.
    let mut expanded = String::new();
    let mut rest = expr;
    while let Some(start) = rest.find('{') {
        expanded.push_str(&rest[..start]);
        rest = &rest[start + 1..];
        let end = rest.find('}').ok_or("Feche a variável com }.")?;
        let name = &rest[..end];
        let value = vars
            .get(name)
            .ok_or(format!("Variável {{{name}}} não cadastrada."))?;
        expanded.push_str(&value.to_string());
        rest = &rest[end + 1..];
    }
    expanded.push_str(rest);
    let chars: Vec<_> = expanded
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| if c == ',' { '.' } else { c })
        .collect();
    // Bound recursive unary and power chains as well as parentheses.
    if chars.len() > 512 {
        return Err("Simplifique a fórmula (até 512 símbolos).".into());
    }
    let mut p = Parser {
        chars,
        pos: 0,
        vars,
        depth: 0,
    };
    let n = p.expr()?;
    if p.pos != p.chars.len() {
        return Err(format!("Símbolo inesperado: {}.", p.peek()));
    }
    if !n.is_finite() || n.abs() > 1e15 {
        return Err("Resultado indefinido ou fora do limite numérico.".into());
    }
    Ok(n)
}

// MoodTest.py:197: remove ,00 (or more zeros) only when no decimal digit follows.
fn remove_zero_decimals(text: &str) -> String {
    let chars: Vec<_> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    let mut after_digit = false;
    while i < chars.len() {
        if chars[i] == ',' && after_digit {
            let mut end = i + 1;
            while chars.get(end) == Some(&'0') {
                end += 1;
            }
            if end > i + 1 && !chars.get(end).is_some_and(char::is_ascii_digit) {
                i = end;
                after_digit = false;
                continue;
            }
        }
        out.push(chars[i]);
        after_digit = chars[i].is_ascii_digit();
        i += 1;
    }
    out
}

pub fn render(text: &str, vars: &BTreeMap<String, f64>) -> Result<String, String> {
    // Question._processar_texto (MoodTest.py:1075), before calculation/substitution.
    let normalized = text
        .replace(r"\(", "")
        .replace(r"\)", "")
        .replace(r"\Omega", "Ω")
        .replace(r"\omega", "ω")
        .replace(r"\ohm", "Ω");
    let mut out = String::new();
    let mut rest = normalized.as_str();
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        rest = &rest[i + 1..];
        let mut depth = 0;
        let mut end = None;
        for (j, c) in rest.char_indices() {
            if c == '{' {
                depth += 1
            } else if c == '}' {
                if depth == 0 {
                    end = Some(j);
                    break;
                } else {
                    depth -= 1
                }
            }
        }
        let j = end.ok_or("Feche os campos com }.")?;
        let inner = &rest[..j];
        if let Some(e) = inner.strip_prefix('=') {
            out.push_str(&fmt(eval(e, vars)?))
        } else {
            out.push_str(&fmt(*vars
                .get(inner)
                .ok_or(format!("Variável {{{inner}}} não cadastrada."))?))
        }
        rest = &rest[j + 1..];
    }
    out.push_str(rest);
    Ok(remove_zero_decimals(&out).trim().to_owned())
}
fn picture_hash(pictures: &[Picture]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for p in pictures {
        p.data.hash(&mut h)
    }
    h.finish()
}
pub fn instance(
    q: &Question,
    values: BTreeMap<String, f64>,
    rng: &mut StdRng,
    shuffle: bool,
) -> Result<Instance, String> {
    let mut answers: Vec<_> = q
        .answers
        .iter()
        .map(|a| {
            Ok(Answer {
                text: render(&a.text, &values)?,
                correct: a.correct,
                pictures: a.pictures.clone(),
            })
        })
        .collect::<Result<_, String>>()?;
    let mut unique = BTreeSet::new();
    for a in &answers {
        if !unique.insert((a.text.trim().to_lowercase(), picture_hash(&a.pictures))) {
            return Err(
                "Duas alternativas produzem o mesmo texto. Ajuste as fórmulas ou os intervalos."
                    .into(),
            );
        }
    }
    if shuffle {
        answers.shuffle(rng)
    }
    Ok(Instance {
        title: q.title.clone(),
        text: render(&q.text, &values)?,
        values,
        pictures: q.pictures.clone(),
        answers,
    })
}
pub fn sample(q: &Question, rng: &mut StdRng) -> BTreeMap<String, f64> {
    q.variables
        .iter()
        .map(|v| (v.name.clone(), v.at(rng.gen_range(0..v.count()))))
        .collect()
}
pub fn preview(q: &Question, seed: u64) -> Result<Instance, String> {
    let mut rng = StdRng::seed_from_u64(seed);
    let values = sample(q, &mut rng);
    instance(q, values, &mut rng, false)
}
pub fn validate(q: &Question) -> Result<String, String> {
    if !q.import_issues.is_empty() {
        return Err(q.import_issues.join("\n"));
    }
    if q.title.trim().is_empty() || q.category.trim().is_empty() || q.text.trim().is_empty() {
        return Err("Preencha título, categoria e enunciado.".into());
    }
    if q.title.len() > 200 || q.category.len() > 100 || q.text.len() > 20000 {
        return Err("Título: até 200 caracteres. Categoria: 100. Enunciado: 20.000.".into());
    }
    if q.pictures.len() > 4 {
        return Err("Use até 4 imagens por questão.".into());
    }
    for pic in q
        .pictures
        .iter()
        .chain(q.answers.iter().flat_map(|a| a.pictures.iter()))
    {
        check_picture(pic)?;
    }
    if q.variables.len() > 12 {
        return Err("Use até 12 variáveis por questão.".into());
    }
    if !(2..=8).contains(&q.answers.len()) || !q.answers.iter().any(|a| a.correct) {
        return Err("Cadastre de 2 a 8 alternativas e marque ao menos uma correta.".into());
    }
    if q.answers
        .iter()
        .any(|a| (a.text.trim().is_empty() && a.pictures.is_empty()) || a.text.len() > 5000)
    {
        return Err("Cada alternativa precisa de um texto (até 5.000 caracteres).".into());
    }
    let mut names = BTreeSet::new();
    let mut count = 1usize;
    for v in &q.variables {
        if v.name.is_empty()
            || !v.name.chars().next().unwrap().is_ascii_alphabetic()
            || !v
                .name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err("Nomes de variável: comece com uma letra, use letras, números e _.".into());
        }
        if !names.insert(v.name.clone()) {
            return Err("Há nomes de variável repetidos.".into());
        }
        if !v.min.is_finite()
            || !v.max.is_finite()
            || v.min > v.max
            || v.min.abs() > 1e9
            || v.max.abs() > 1e9
            || v.decimals > 6
            || v.values.len() > 10000
            || v.values.iter().any(|x| !x.is_finite() || x.abs() > 1e9)
        {
            return Err(format!(
                "Intervalo inválido para {}. Mínimo ≤ máximo; até 6 casas; valores até 1 bilhão.",
                v.name
            ));
        }
        let scale = 10f64.powi(v.decimals as i32);
        if v.values.is_empty()
            && ((v.min * scale - v.min.mul_add(scale, 0.0).round()).abs() > 1e-5
                || (v.max * scale - (v.max * scale).round()).abs() > 1e-5)
        {
            return Err(format!(
                "A precisão de {} não comporta o mínimo/máximo.",
                v.name
            ));
        }
        count = count.saturating_mul(v.count());
    }
    let exhaustive = count <= 10000;
    let tests = if exhaustive { count } else { 256 };
    let mut rng = StdRng::seed_from_u64(42);
    let signatures: Vec<_> = q
        .answers
        .iter()
        .map(|a| picture_hash(&a.pictures))
        .collect();
    for i in 0..tests {
        let values = if exhaustive {
            let mut n = i;
            q.variables
                .iter()
                .map(|v| {
                    let x = v.at(n % v.count());
                    n /= v.count();
                    (v.name.clone(), x)
                })
                .collect()
        } else if i < 2 {
            q.variables
                .iter()
                .map(|v| (v.name.clone(), v.at(if i == 0 { 0 } else { v.count() - 1 })))
                .collect()
        } else {
            sample(q, &mut rng)
        };
        let check = (|| -> Result<(), String> {
            render(&q.text, &values)?;
            let mut unique = BTreeSet::new();
            for (j, a) in q.answers.iter().enumerate() {
                let s = render(&a.text, &values)?;
                if !unique.insert((s.trim().to_lowercase(), signatures[j])) {
                    return Err("Duas alternativas produzem o mesmo texto. Ajuste as fórmulas ou os intervalos.".into());
                }
            }
            Ok(())
        })();
        check.map_err(|e| {
            format!(
                "{e}\nValores do teste: {}",
                values
                    .iter()
                    .map(|(k, v)| format!("{k}={}", fmt(*v)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
    }
    Ok(if exhaustive {
        format!("Validada em {tests} combinações. Sem erros nas fórmulas.")
    } else {
        "Pré-validada em 256 amostras; o intervalo completo é maior. Cada prova será validada antes de salvar.".into()
    })
}
pub fn categories(bank: &Bank) -> Vec<String> {
    let mut labels = BTreeSet::new();
    for path in bank
        .category_labels
        .iter()
        .chain(bank.questions.iter().map(|q| &q.category))
    {
        if !path.is_empty() {
            labels.insert(path.clone());
        }
        for (index, _) in path.match_indices('/') {
            if index > 0 {
                labels.insert(path[..index].to_owned());
            }
        }
    }
    let mut labels: Vec<_> = labels.into_iter().collect();
    labels.sort_by(|a, b| a.split('/').cmp(b.split('/')));
    labels
}
pub fn category_matches(category: &str, parent: &str) -> bool {
    category == parent
        || category
            .strip_prefix(parent)
            .is_some_and(|s| s.starts_with('/'))
}
pub fn new_category(name: &str, parent: &str, existing: &[String]) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() || name.contains('/') || name.chars().any(char::is_control) {
        return Err(
            "Informe um nome sem barras. Escolha a categoria principal no campo abaixo.".into(),
        );
    }
    if !parent.is_empty() && !existing.iter().any(|c| c == parent) {
        return Err("Selecione uma categoria principal existente.".into());
    }
    let path = if parent.is_empty() {
        name.to_owned()
    } else {
        format!("{parent}/{name}")
    };
    if path == "Todas" {
        return Err("Escolha outro nome: Todas é usado para exibir o banco completo.".into());
    }
    if path.len() > 100 {
        return Err("O caminho da categoria deve ter até 100 bytes.".into());
    }
    if existing
        .iter()
        .any(|c| c.to_lowercase() == path.to_lowercase())
    {
        return Err("Esta categoria já existe.".into());
    }
    Ok(path)
}
pub fn renamed_path(path: &str, old: &str, new: &str) -> String {
    if category_matches(path, old) {
        format!("{new}{}", &path[old.len()..])
    } else {
        path.to_owned()
    }
}
pub fn rename_category(
    bank: &mut Bank,
    old: &str,
    name: &str,
    parent: &str,
) -> Result<String, String> {
    let existing = categories(bank);
    if !existing.iter().any(|c| c == old) {
        return Err("Selecione uma categoria existente.".into());
    }
    if category_matches(parent, old) {
        return Err(
            "Uma categoria não pode ficar dentro dela mesma ou de suas subcategorias.".into(),
        );
    }
    let outside: Vec<_> = existing
        .iter()
        .filter(|c| !category_matches(c, old))
        .cloned()
        .collect();
    let path = new_category(name, parent, &outside)?;
    let labels: Vec<_> = existing
        .iter()
        .map(|c| renamed_path(c, old, &path))
        .collect();
    if labels.iter().any(|c| c.len() > 100) {
        return Err("Uma subcategoria excederia o limite de 100 bytes.".into());
    }
    for q in &mut bank.questions {
        q.category = renamed_path(&q.category, old, &path);
    }
    bank.category_labels = labels;
    Ok(path)
}
pub fn selected_bank(bank: &Bank, ids: &BTreeSet<u64>) -> Bank {
    let mut selection = Bank {
        version: bank.version,
        questions: bank
            .questions
            .iter()
            .filter(|q| ids.contains(&q.id))
            .cloned()
            .collect(),
        category_labels: vec![],
    };
    selection.category_labels = categories(&selection);
    selection
}
pub fn transfer_questions(
    bank: &mut Bank,
    ids: &BTreeSet<u64>,
    destination: &str,
    copy: bool,
) -> Result<usize, String> {
    if !categories(bank).iter().any(|c| c == destination) {
        return Err("Selecione a categoria de destino.".into());
    }
    let mut questions = selected_bank(bank, ids).questions;
    if questions.is_empty() {
        return Err("Selecione pelo menos uma questão.".into());
    }
    // Keep categories even when their last question is moved.
    let labels = categories(bank);
    let count = questions.len();
    if copy {
        if bank.questions.len() + count > 10000 {
            return Err("A cópia excederia o limite de 10.000 questões.".into());
        }
        let mut next = bank.questions.iter().map(|q| q.id).max().unwrap_or(0);
        next.checked_add(count as u64)
            .ok_or("Não há identificadores disponíveis.")?;
        for q in &mut questions {
            next += 1;
            q.id = next;
            q.category = destination.to_owned();
        }
        bank.questions.extend(questions);
    } else {
        for q in &mut bank.questions {
            if ids.contains(&q.id) {
                q.category = destination.to_owned();
            }
        }
    }
    bank.category_labels = labels;
    Ok(count)
}
pub fn generate(
    bank: &Bank,
    plan: &[(String, usize)],
    names: &str,
    seed: u64,
    title: &str,
    institution: &str,
    teacher: &str,
    date: &str,
    class_name: &str,
) -> Result<Batch, String> {
    let names: Vec<_> = names
        .lines()
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .collect();
    if names.is_empty() {
        return Err("Adicione ao menos um aluno, um nome por linha.".into());
    }
    if names.len() > 1000 {
        return Err("Gere até 1.000 alunos por lote.".into());
    }
    if names.iter().collect::<BTreeSet<_>>().len() != names.len() {
        return Err("Há nomes repetidos. Acrescente a matrícula para diferenciá-los.".into());
    }
    if title.trim().is_empty() {
        return Err("Dê um título à prova.".into());
    }
    if plan.iter().map(|p| p.1).sum::<usize>() == 0 {
        return Err("Escolha a quantidade de questões por categoria.".into());
    }
    if plan.iter().map(|p| p.1).sum::<usize>() > 100 {
        return Err("Use até 100 questões por prova.".into());
    }
    for (cat, n) in plan {
        let qs: Vec<_> = bank
            .questions
            .iter()
            .filter(|q| &q.category == cat)
            .collect();
        if *n > qs.len() {
            return Err(format!(
                "{cat}: há {} questões, mas foram pedidas {n}.",
                qs.len()
            ));
        }
        if *n > 0 {
            for q in qs {
                validate(q).map_err(|e| format!("{}: {e}", q.title))?;
            }
        }
    }
    let mut rng = StdRng::seed_from_u64(seed);
    let mut students = vec![];
    for name in names {
        let mut questions = vec![];
        for (cat, n) in plan {
            let mut qs: Vec<_> = bank
                .questions
                .iter()
                .filter(|q| &q.category == cat)
                .collect();
            qs.shuffle(&mut rng);
            for q in qs.into_iter().take(*n) {
                let values = sample(q, &mut rng);
                questions.push(
                    instance(q, values, &mut rng, true).map_err(|e| format!("{}: {e}", q.title))?,
                );
            }
        }
        students.push(StudentExam {
            name: name.into(),
            questions,
        });
    }
    Ok(Batch {
        version: 1,
        seed,
        plan: plan.to_vec(),
        title: title.into(),
        institution: institution.into(),
        teacher: teacher.into(),
        date: date.into(),
        class_name: class_name.into(),
        students,
    })
}
pub fn save_bank(path: &Path, bank: &Bank) -> Result<(), String> {
    let data = serde_json::to_vec_pretty(bank).map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, data).map_err(|e| e.to_string())?;
    // On Windows rename replaces an existing file atomically.
    std::fs::rename(tmp, path).map_err(|e| e.to_string())
}
pub fn read_bank(path: &Path) -> Result<Bank, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    if bytes.len() > 20_000_000 {
        return Err("O banco excede 20 MB.".into());
    }
    let bank: Bank =
        serde_json::from_slice(&bytes).map_err(|e| format!("Banco JSON inválido: {e}"))?;
    if bank.version != 1 {
        return Err("Versão de banco não suportada.".into());
    }
    if bank.questions.len() > 10000 {
        return Err("Banco com mais de 10.000 questões.".into());
    }
    let mut ids = BTreeSet::new();
    for q in &bank.questions {
        if !ids.insert(q.id) {
            return Err("Banco com identificadores repetidos.".into());
        }
    }
    Ok(bank)
}
pub fn examples() -> Bank {
    fn q(
        id: u64,
        title: &str,
        cat: &str,
        text: &str,
        vars: &[(&str, f64, f64)],
        answers: &[&str],
    ) -> Question {
        Question {
            import_issues: vec![],
            source_xml: None,
            id,
            title: title.into(),
            category: cat.into(),
            text: text.into(),
            pictures: vec![],
            variables: vars
                .iter()
                .map(|(n, a, b)| Variable {
                    name: n.to_string(),
                    min: *a,
                    max: *b,
                    decimals: 0,
                    values: vec![],
                })
                .collect(),
            answers: answers
                .iter()
                .enumerate()
                .map(|(i, t)| Answer {
                    text: t.to_string(),
                    correct: i == 0,
                    pictures: vec![],
                })
                .collect(),
        }
    }
    Bank{version:1,category_labels:vec![],questions:vec![
        q(1,"Velocidade média","Mecânica","Um ciclista percorre {d} metros em {t} segundos. Qual é sua velocidade média?",&[("d",100.,300.),("t",10.,30.)],&["{={d}/{t}} m/s","{={d}*{t}} m/s","{={d}/{t}+5} m/s","{={d}/{t}/2} m/s"]),
        q(2,"Segunda lei de Newton","Mecânica","Um corpo de massa {m} kg está sujeito a uma aceleração de {a} m/s². Qual é a força resultante?",&[("m",2.,20.),("a",2.,8.)],&["{={m}*{a}} N","{={m}*{a}+10} N","{={m}*{a}/2} N","{={m}*{a}+5} N"]),
        q(3,"Energia cinética","Mecânica","Um objeto de massa {m} kg se move a {v} m/s. Calcule sua energia cinética.",&[("m",2.,12.),("v",2.,10.)],&["{={m}*{v}^2/2} J","{={m}*{v}^2} J","{={m}*{v}^2/4} J","{={m}*{v}^2*1.5} J"]),
        q(4,"Lei de Ohm","Eletricidade","Um resistor de {r} ohms é submetido a uma tensão de {u} V. Qual é a corrente elétrica?",&[("r",10.,50.),("u",20.,100.)],&["{={u}/{r}} A","{={u}*{r}} A","{={u}/{r}+2} A","{={u}/{r}/2} A"]),
        q(5,"Potência elétrica","Eletricidade","Um aparelho opera com tensão de {u} V e corrente de {i} A. Qual é sua potência?",&[("u",100.,240.),("i",2.,8.)],&["{={u}*{i}} W","{={u}*{i}/2} W","{={u}*{i}+100} W","{={u}*{i}*2} W"]),
        q(6,"Área do retângulo","Matemática","Um terreno retangular tem {b} m de base e {h} m de altura. Qual é sua área?",&[("b",10.,30.),("h",5.,15.)],&["{={b}*{h}} m²","{={b}*{h}*2} m²","{={b}*{h}/2} m²","{={b}*{h}+10} m²"])
    ]}
}

pub fn check_picture(p: &Picture) -> Result<(), String> {
    if p.data.len() > 5_000_000 {
        return Err("Cada imagem pode ter até 5 MB.".into());
    }
    let reader = image::ImageReader::new(std::io::Cursor::new(&p.data))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    let (w, h) = reader
        .into_dimensions()
        .map_err(|e| format!("Imagem inválida: {e}"))?;
    if w == 0 || h == 0 || w > 8000 || h > 8000 || w as u64 * h as u64 > 20_000_000 {
        return Err("Imagem muito grande: até 20 megapixels e 8.000 px por lado.".into());
    }
    Ok(())
}
fn decode64(s: &str) -> Result<Vec<u8>, String> {
    let mut out = vec![];
    let mut buf = 0u32;
    let mut bits = 0;
    for c in s.chars() {
        let n = match c {
            'A'..='Z' => c as u8 - b'A',
            'a'..='z' => c as u8 - b'a' + 26,
            '0'..='9' => c as u8 - b'0' + 52,
            '+' => 62,
            '/' => 63,
            '=' => break,
            c if c.is_whitespace() => continue,
            _ => return Err("Imagem com base64 inválido.".into()),
        };
        buf = (buf << 6) | n as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Ok(out)
}
fn plain(html: &str) -> Result<String, String> {
    let mut s = String::new();
    let mut tag = String::new();
    let mut inside = false;
    for c in html.chars() {
        if c == '<' {
            inside = true;
            tag.clear()
        } else if c == '>' {
            inside = false;
            if ["br", "/p", "/div", "/li"]
                .iter()
                .any(|v| tag.to_lowercase().starts_with(v))
            {
                s.push('\n')
            }
        } else if inside {
            tag.push(c)
        } else {
            s.push(c)
        }
    }
    Ok(s.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .trim()
        .to_owned())
}
#[derive(Default)]
struct XmlNode {
    tag: String,
    attrs: BTreeMap<String, String>,
    text: String,
    children: Vec<XmlNode>,
}
impl XmlNode {
    fn xml(&self) -> String {
        fn escape(s: &str) -> String {
            s.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
        }
        let attrs = self
            .attrs
            .iter()
            .map(|(k, v)| format!(" {k}=\"{}\"", escape(v)))
            .collect::<String>();
        format!(
            "<{}{attrs}>{}{}</{}>",
            self.tag,
            escape(&self.text),
            self.children.iter().map(Self::xml).collect::<String>(),
            self.tag
        )
    }
    fn child(&self, tag: &str) -> Option<&Self> {
        self.children.iter().find(|x| x.tag == tag)
    }
    fn nested(&self, tag: &str) -> String {
        self.child(tag)
            .and_then(|x| x.child("text"))
            .map(|x| x.text.clone())
            .unwrap_or_default()
    }
    fn find<'a>(&'a self, tag: &str, out: &mut Vec<&'a Self>) {
        for c in &self.children {
            if c.tag == tag {
                out.push(c)
            }
            c.find(tag, out)
        }
    }
}
fn xml_pictures(node: &XmlNode, issues: &mut Vec<String>) -> Vec<Picture> {
    let mut pictures = vec![];
    for file in node.children.iter().filter(|x| x.tag == "file") {
        let name = file
            .attrs
            .get("name")
            .cloned()
            .unwrap_or_else(|| "Imagem".into());
        match decode64(&file.text) {
            Ok(data) => {
                let picture = Picture { name, data };
                if let Err(e) = check_picture(&picture) {
                    issues.push(e);
                }
                pictures.push(picture);
            }
            Err(e) => issues.push(format!("{name}: {e}")),
        }
    }
    if node
        .child("text")
        .map(|x| x.text.to_lowercase().contains("<img"))
        .unwrap_or(false)
        && pictures.is_empty()
    {
        issues.push(
            "Imagem externa não incluída no XML. Anexe a imagem na edição da questão.".into(),
        );
    }
    pictures
}
pub fn import_xml(path: &Path) -> Result<Bank, String> {
    use xml::reader::{EventReader, XmlEvent};
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    if bytes.len() > 20_000_000 {
        return Err("O XML excede 20 MB.".into());
    }
    let mut stack = vec![XmlNode::default()];
    for event in EventReader::new(bytes.as_slice()) {
        match event.map_err(|e| e.to_string())? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => {
                if stack.len() > 100 {
                    return Err("XML muito complexo.".into());
                }
                stack.push(XmlNode {
                    tag: name.local_name,
                    attrs: attributes
                        .into_iter()
                        .map(|a| (a.name.local_name, a.value))
                        .collect(),
                    ..Default::default()
                });
            }
            XmlEvent::Characters(s) | XmlEvent::CData(s) => {
                stack.last_mut().unwrap().text.push_str(&s)
            }
            XmlEvent::EndElement { .. } => {
                if stack.len() > 1 {
                    let n = stack.pop().unwrap();
                    stack.last_mut().unwrap().children.push(n);
                }
            }
            _ => {}
        }
    }
    let root = stack.pop().unwrap();
    let mut nodes = vec![];
    root.find("question", &mut nodes);
    let mut category = "Importadas".to_owned();
    let mut questions = vec![];
    for n in nodes {
        let kind = n.attrs.get("type").map(String::as_str).unwrap_or("");
        if kind == "category" {
            category = n
                .nested("category")
                .rsplit('/')
                .next()
                .unwrap_or("Importadas")
                .to_owned();
            continue;
        }
        let mut import_issues = vec![];
        if !["calculatedmulti", "multichoice"].contains(&kind) {
            import_issues.push(format!("Tipo Moodle '{kind}' não suportado. Adapte a questão no editor; o XML original foi preservado."));
        }
        let pictures = n
            .child("questiontext")
            .map(|node| xml_pictures(node, &mut import_issues))
            .unwrap_or_default();
        let mut variables: Vec<Variable> = vec![];
        let mut defs = vec![];
        // MoodTest.py carregar_questoes, source offset 0xc549: only
        // calculatedmulti uses datasets; trim individual values before parsing.
        if kind == "calculatedmulti" {
            n.find("dataset_definition", &mut defs);
        }
        for d in defs {
            let name = d.nested("name");
            // carregar_questoes (0xc549), unnamed guard at source offset 0xcba2.
            if name.is_empty() {
                continue;
            }
            let mut vals = vec![];
            d.find("value", &mut vals);
            let values: Vec<f64> = vals
                .iter()
                .filter_map(|x| match x.text.trim().replace(',', ".").parse::<f64>() {
                    Ok(v) if v.is_finite() => Some(v),
                    _ => {
                        import_issues.push(format!(
                            "Dataset {name}: valor inválido '{}'. Revise os valores.",
                            x.text
                        ));
                        None
                    }
                })
                .collect();
            if values.is_empty() {
                import_issues.push(format!(
                    "Dataset {name} sem valores. Defina um intervalo no editor."
                ));
            }
            let variable = Variable {
                name,
                min: values.iter().copied().reduce(f64::min).unwrap_or(0.),
                max: values.iter().copied().reduce(f64::max).unwrap_or(0.),
                decimals: 2,
                values,
            };
            // Original dictionary assignment at 0xcfc0: later definitions replace
            // earlier ones without changing the name's first insertion position.
            if let Some(existing) = variables.iter_mut().find(|v| v.name == variable.name) {
                *existing = variable;
            } else {
                variables.push(variable);
            }
        }
        let answers = n
            .children
            .iter()
            .filter(|x| x.tag == "answer")
            .map(|a| {
                let ap = xml_pictures(a, &mut import_issues);
                let raw = a.child("text").map(|x| x.text.clone()).unwrap_or_default();
                let fraction = match a.attrs.get("fraction").and_then(|x| x.trim().parse::<f64>().ok()) {
                    Some(v) if v.is_finite() => v,
                    _ => { import_issues.push("Pontuação de alternativa inválida. Revise quais respostas são corretas.".into()); 0. }
                };
                Ok(Answer {
                    text: plain(&raw)?,
                    correct: fraction > 0.,
                    pictures: ap,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let q = Question {
            id: questions.len() as u64 + 1,
            title: n.nested("name"),
            // carregar_questoes (0xc549), category branch at 0xc680:
            // preserve an explicit category on the question, including its full path.
            category: n
                .child("category")
                .map(|_| n.nested("category"))
                .unwrap_or_else(|| category.clone()),
            text: plain(&n.nested("questiontext"))?,
            pictures,
            variables,
            answers,
            import_issues,
            source_xml: Some(n.xml()),
        };
        questions.push(q);
    }
    if questions.is_empty() {
        return Err("Nenhuma questão encontrada no XML.".into());
    }
    Ok(Bank {
        version: 1,
        category_labels: vec![],
        questions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn categories_and_batch_changes_survive_reopening() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("checks/category-actions");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bank.json");
        let mut bank = examples();
        let count = bank.questions.len();
        for name in ["Nova", "Outra", "Vazia"] {
            let category = new_category(name, "", &categories(&bank)).unwrap();
            bank.category_labels.push(category);
            bank.category_labels = categories(&bank);
            save_bank(&path, &bank).unwrap();
            bank = read_bank(&path).unwrap();
        }
        assert_eq!(bank.questions.len(), count);
        assert!(categories(&bank).contains(&"Nova".into()));
        assert!(categories(&bank).contains(&"Outra".into()));
        let ids = BTreeSet::from([bank.questions[0].id, bank.questions[1].id]);
        let originals: Vec<_> = bank.questions.iter().take(2).cloned().collect();
        assert_eq!(
            transfer_questions(&mut bank, &ids, "Nova", true).unwrap(),
            2
        );
        assert_eq!(bank.questions.len(), count + 2);
        for (q, original) in bank.questions.iter().zip(&originals) {
            assert_eq!(q.category, original.category);
        }
        assert_eq!(
            bank.questions
                .iter()
                .map(|q| q.id)
                .collect::<BTreeSet<_>>()
                .len(),
            count + 2
        );
        assert_eq!(
            transfer_questions(&mut bank, &ids, "Outra", false).unwrap(),
            2
        );
        let selection = selected_bank(&bank, &ids);
        assert_eq!(selection.questions.len(), 2);
        assert!(selection.questions.iter().all(|q| q.category == "Outra"));
        save_bank(&path, &selection).unwrap();
        assert_eq!(read_bank(&path).unwrap().questions.len(), 2);
        // Removing selected questions must preserve other copies and categories.
        bank.questions.retain(|q| !ids.contains(&q.id));
        save_bank(&path, &bank).unwrap();
        bank = read_bank(&path).unwrap();
        assert_eq!(bank.questions.len(), count);
        assert_eq!(
            bank.questions
                .iter()
                .filter(|q| q.category == "Nova")
                .count(),
            2
        );
        assert!(categories(&bank).contains(&"Outra".into()));
        assert!(categories(&bank).contains(&"Vazia".into()));
    }
    #[test]
    fn category_edits_follow_descendants_and_reject_invalid_moves_atomically() {
        let mut bank = examples();
        bank.category_labels = vec![
            "Física/Óptica/Lentes".into(),
            "Destino".into(),
            "Física aplicada".into(),
        ];
        bank.questions[0].category = "Física/Óptica/Lentes".into();
        rename_category(&mut bank, "Física/Óptica", "Luz", "Destino").unwrap();
        assert_eq!(bank.questions[0].category, "Destino/Luz/Lentes");
        assert!(categories(&bank).contains(&"Física aplicada".into()));
        let before = serde_json::to_vec(&bank).unwrap();
        assert!(rename_category(&mut bank, "Destino", "Loop", "Destino/Luz").is_err());
        assert!(rename_category(&mut bank, "Destino", "Física aplicada", "").is_err());
        assert!(rename_category(&mut bank, "Destino", &"x".repeat(95), "").is_err());
        let id = bank.questions[0].id;
        assert!(transfer_questions(&mut bank, &BTreeSet::from([id]), "Ausente", false).is_err());
        assert_eq!(serde_json::to_vec(&bank).unwrap(), before);
        rename_category(&mut bank, "Destino", "Ciências", "").unwrap();
        assert_eq!(bank.questions[0].category, "Ciências/Luz/Lentes");
    }
    #[test]
    fn hierarchical_categories_survive_export_and_legacy_banks() {
        let mut bank: Bank = serde_json::from_str(r#"{"version":1,"questions":[]}"#).unwrap();
        assert!(categories(&bank).is_empty());
        bank.category_labels = vec![
            "Física".into(),
            "Física/Óptica/Lentes".into(),
            "Física aplicada".into(),
        ];
        let mut q = examples().questions.remove(0);
        q.category = "Física/Mecânica".into();
        bank.questions.push(q);
        let restored: Bank = serde_json::from_slice(&serde_json::to_vec(&bank).unwrap()).unwrap();
        assert_eq!(
            categories(&restored),
            vec![
                "Física",
                "Física/Mecânica",
                "Física/Óptica",
                "Física/Óptica/Lentes",
                "Física aplicada"
            ]
        );
        assert!(category_matches("Física/Óptica/Lentes", "Física/Óptica"));
        assert!(!category_matches("Física aplicada", "Física"));
        bank.questions.clear();
        assert!(categories(&bank).contains(&"Física/Óptica/Lentes".into()));
    }
    #[test]
    fn category_creation_validates_parent_name_and_duplicates() {
        let existing = vec!["Física".into(), "Física/Óptica".into()];
        assert_eq!(
            new_category("  Lentes  ", "Física/Óptica", &existing).unwrap(),
            "Física/Óptica/Lentes"
        );
        for (name, parent) in [
            (" ", ""),
            ("A/B", ""),
            ("física", ""),
            ("Óptica", "Física"),
            ("Lentes", "Inexistente"),
            ("Todas", ""),
        ] {
            assert!(new_category(name, parent, &existing).is_err());
        }
        assert!(new_category(&"x".repeat(101), "", &existing).is_err());
    }
    #[test]
    fn imports_mixed_errors_and_reopens_saved_questions() {
        let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("checks/tolerant-import");
        std::fs::create_dir_all(&folder).unwrap();
        let xml = folder.join("mixed.xml");
        std::fs::write(&xml, r#"<quiz>
          <question type="category"><category><text>$course$/Importadas</text></category></question>
          <question type="calculatedmulti"><name><text>Formula</text></name><questiontext><text>{=1/0}</text></questiontext><answer fraction="100"><text>A</text></answer><answer fraction="0"><text>B</text></answer></question>
          <question type="calculatedmulti"><name><text>Dataset</text></name><questiontext><text>{x}</text></questiontext><dataset_definitions><dataset_definition><name><text>x</text></name><dataset_items><dataset_item><value>NaN</value></dataset_item><dataset_item><value>erro</value></dataset_item></dataset_items></dataset_definition></dataset_definitions></question>
          <question type="multichoice"><name><text>Imagem</text></name><questiontext><text><![CDATA[<img src="missing.png">]]></text><file name="bad.png">???</file></questiontext></question>
          <question type="essay"><name><text>Outro tipo</text></name><questiontext><text>Explique</text></questiontext></question>
          <question type="multichoice"><name><text>Valida</text></name><questiontext><text>Escolha A</text></questiontext><answer fraction="100"><text>A</text></answer><answer fraction="0"><text>B</text></answer></question>
        </quiz>"#).unwrap();
        let mut bank = import_xml(&xml).unwrap();
        assert_eq!(bank.questions.len(), 5);
        assert_eq!(
            bank.questions
                .iter()
                .filter(|q| validate(q).is_err())
                .count(),
            4
        );
        assert!(validate(&bank.questions[4]).is_ok());
        assert!(bank.questions[2]
            .source_xml
            .as_ref()
            .unwrap()
            .contains("???"));
        assert!(bank.questions[3]
            .source_xml
            .as_ref()
            .unwrap()
            .contains("essay"));
        let json = folder.join("bank.json");
        save_bank(&json, &bank).unwrap();
        let reopened = read_bank(&json).unwrap();
        assert_eq!(reopened.questions.len(), 5);
        assert_eq!(
            reopened.questions[1].import_issues,
            bank.questions[1].import_issues
        );
        assert!(validate(&reopened.questions[0]).is_err());
        // Fixing the formula clears its error without losing the original source.
        bank.questions[0].text = "Escolha A".into();
        assert!(validate(&bank.questions[0]).is_ok());
        // Questions produced in the editor can also be stored with missing fields.
        let mut draft = examples().questions.remove(0);
        draft.id = 99;
        draft.title.clear();
        draft.answers.clear();
        bank.questions.push(draft);
        save_bank(&json, &bank).unwrap();
        let reopened = read_bank(&json).unwrap();
        assert_eq!(reopened.questions.len(), 6);
        assert!(validate(&reopened.questions[5]).is_err());
        std::fs::write(&xml, "<quiz><question>").unwrap();
        assert!(import_xml(&xml).is_err());
    }
    #[test]
    fn numeric_grammar() {
        let v = BTreeMap::from([("a".into(), 4.)]);
        assert_eq!(eval("-2^2 + sqrt({a})", &v).unwrap(), -2.);
        assert_eq!(eval("2**3**2", &v).unwrap(), 512.);
        assert!(eval("1/0", &v).is_err());
        assert!(eval("sqrt(-1)", &v).is_err());
        assert!(eval("system(1)", &v).is_err());
        assert!(eval("missing+1", &v).is_err());
    }
    #[test]
    fn formatting_and_blocks() {
        assert_eq!(fmt(1234.5), "1.234,50");
        assert_eq!(fmt(-1.125), "-1,13");
        assert_eq!(fmt(10.), "10");
        assert_eq!(fmt(1.005), "1,01");
        assert_eq!(fmt(-1.005), "-1,01");
        assert_eq!(
            render("Valor {= {a}^2/2} N", &BTreeMap::from([("a".into(), 4.)])).unwrap(),
            "Valor 8 N"
        );
    }
    #[test]
    fn matches_original_numeric_text_results() {
        // Previously extracted from the original's isolated
        // numeric functions, without importing its GUI or installer.
        let mechanics: serde_json::Value =
            serde_json::from_str(include_str!("../mechanics.json")).unwrap();
        let cases = mechanics["original"]["reference_cases"].as_array().unwrap();
        assert!(!cases.is_empty());
        for case in cases {
            let variables: BTreeMap<String, f64> =
                serde_json::from_value(case["variables"].clone()).unwrap();
            let text = case["text"].as_str().unwrap();
            assert_eq!(
                render(text, &variables).unwrap(),
                case["expected"].as_str().unwrap(),
                "original source case: {text}"
            );
        }
    }
    #[test]
    fn matches_original_expression_domains() {
        let mechanics: serde_json::Value =
            serde_json::from_str(include_str!("../mechanics.json")).unwrap();
        for case in mechanics["original"]["expression_domain_cases"]
            .as_array()
            .unwrap()
        {
            let expression = case["expression"].as_str().unwrap();
            let result = eval(expression, &BTreeMap::new());
            if case.get("error").is_some() {
                assert!(
                    result.is_err(),
                    "original rejects {expression}; rebuild returned {result:?}"
                );
            } else {
                assert_eq!(
                    result.unwrap(),
                    case["value"].as_f64().unwrap(),
                    "{expression}"
                );
            }
        }
    }
    #[test]
    fn dataset_definitions_match_original() {
        let mechanics: serde_json::Value =
            serde_json::from_str(include_str!("../mechanics.json")).unwrap();
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("checks/original-dataset-definitions-test");
        std::fs::create_dir_all(&dir).unwrap();
        for case in mechanics["original"]["dataset_definition_cases"]
            .as_array()
            .unwrap()
        {
            let path = dir.join("datasets.xml");
            std::fs::write(&path, case["xml"].as_str().unwrap()).unwrap();
            let bank = import_xml(&path).unwrap();
            let json = dir.join("bank.json");
            save_bank(&json, &bank).unwrap();
            let reopened = read_bank(&json).unwrap();
            let q = &reopened.questions[0];
            let actual: Vec<_> = q
                .variables
                .iter()
                .map(|v| serde_json::json!({"name": v.name, "values": v.values}))
                .collect();
            assert_eq!(
                serde_json::json!(actual),
                case["expected"],
                "{}",
                case["label"]
            );
            assert!(validate(q).is_ok(), "{}", case["label"]);
            let generated = preview(q, 42).unwrap();
            assert!(!generated.text.contains('{'), "{}", case["label"]);
        }
    }
    #[test]
    fn original_dataset_names_and_question_category() {
        let mechanics: serde_json::Value =
            serde_json::from_str(include_str!("../mechanics.json")).unwrap();
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("checks/original-category-test");
        std::fs::create_dir_all(&dir).unwrap();
        for case in mechanics["original"]["question_category_cases"]
            .as_array()
            .unwrap()
        {
            let path = dir.join("category.xml");
            std::fs::write(&path, case["xml"].as_str().unwrap()).unwrap();
            let bank = import_xml(&path).unwrap();
            assert_eq!(
                bank.questions[0].category,
                case["expected"].as_str().unwrap()
            );
        }
        let mut q = examples().questions.remove(0);
        q.text = "Valor {=pi+e+sin+sqrt(9)}".into();
        q.variables = ["pi", "e", "sin"]
            .into_iter()
            .map(|name| Variable {
                name: name.into(),
                min: 2.,
                max: 2.,
                decimals: 0,
                values: vec![2.],
            })
            .collect();
        q.answers[0].text = "{=pi+e+sin+sqrt(9)}".into();
        q.answers[1].text = "Outra resposta".into();
        q.answers.truncate(2);
        assert!(
            validate(&q).is_ok(),
            "original permits dataset names matching constants/functions"
        );
    }
    #[test]
    fn validation_catches_hidden_zero() {
        let mut q = examples().questions.remove(0);
        q.variables[1].min = 0.;
        assert!(validate(&q).is_err());
        q.variables[1].min = 10.;
        q.answers[1] = q.answers[0].clone();
        assert!(validate(&q).is_err());
    }
    #[test]
    fn persistence_and_moodle_import() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("checks/model-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bank.json");
        let mut b = examples();
        save_bank(&path, &b).unwrap();
        b.questions.remove(0);
        save_bank(&path, &b).unwrap();
        assert_eq!(read_bank(&path).unwrap().questions.len(), 5);
        let xml = r#"<quiz><question type="category"><category><text>$course$/Física</text></category></question><question type="calculatedmulti"><name><text>Teste Moodle</text></name><questiontext><text>&lt;p&gt;Valor {x}&lt;/p&gt;</text></questiontext><answer fraction="100"><text>{={x}*2}</text></answer><answer fraction="0"><text>{={x}*3}</text></answer><dataset_definitions><dataset_definition><name><text>x</text></name><dataset_items><dataset_item><value>2</value></dataset_item><dataset_item><value>4</value></dataset_item></dataset_items></dataset_definition></dataset_definitions></question></quiz>"#;
        let path = dir.join("moodle.xml");
        std::fs::write(&path, xml).unwrap();
        let b = import_xml(&path).unwrap();
        assert_eq!(b.questions[0].category, "Física");
        assert_eq!(b.questions[0].variables[0].values, vec![2., 4.]);
        assert_eq!(b.questions[0].text, "Valor {x}");
        assert!(b.questions[0].answers[0].correct);
    }
    #[test]
    fn reproducible_and_correct() {
        let b = examples();
        for q in &b.questions {
            validate(q).unwrap();
        }
        let p = vec![("Mecânica".into(), 2)];
        let a = generate(&b, &p, "Ana\nBia", 45, "Prova", "", "", "", "").unwrap();
        let c = generate(&b, &p, "Ana\nBia", 45, "Prova", "", "", "", "").unwrap();
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&c).unwrap()
        );
        for s in a.students {
            assert_eq!(s.questions.len(), 2);
            assert_ne!(s.questions[0].title, s.questions[1].title);
            for q in s.questions {
                assert_eq!(q.answers.iter().filter(|a| a.correct).count(), 1);
                let expected = if q.title == "Velocidade média" {
                    q.values["d"] / q.values["t"]
                } else if q.title == "Segunda lei de Newton" {
                    q.values["m"] * q.values["a"]
                } else {
                    q.values["m"] * q.values["v"].powi(2) / 2.
                };
                assert!(q
                    .answers
                    .iter()
                    .find(|a| a.correct)
                    .unwrap()
                    .text
                    .starts_with(&fmt(expected)));
            }
        }
    }
    #[test]
    fn moodle_original_dataset_and_score_rules() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("checks/original-import-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("moodle.xml");
        let mechanics: serde_json::Value =
            serde_json::from_str(include_str!("../mechanics.json")).unwrap();
        for case in mechanics["original"]["answer_score_cases"]
            .as_array()
            .unwrap()
        {
            let scores = case["fractions"].as_array().unwrap();
            let answers: String = scores
                .iter()
                .enumerate()
                .map(|(i, score)| {
                    format!(
                        "<answer fraction=\"{}\"><text>Alternativa {i}</text></answer>",
                        score.as_str().unwrap()
                    )
                })
                .collect();
            // The bank requires a correct choice. Add one after the oracle cases,
            // so empty oracle results still exercise the importer's score parsing.
            let answers = format!("{answers}<answer fraction=\"100\"><text>Final</text></answer>");
            for kind in ["calculatedmulti", "multichoice"] {
                let xml = format!(
                    r#"<quiz><question type="{kind}"><name><text>Regras originais</text></name><questiontext><text>Escolha</text></questiontext>{answers}<dataset_definitions><dataset_definition><name><text>x</text></name><dataset_items><dataset_item><value> 2,5 </value></dataset_item><dataset_item><value>2,5</value></dataset_item><dataset_item><value> 4 </value></dataset_item><dataset_item><value>invalid</value></dataset_item></dataset_items></dataset_definition></dataset_definitions></question></quiz>"#
                );
                std::fs::write(&path, xml).unwrap();
                let bank = import_xml(&path).unwrap();
                let q = &bank.questions[0];
                let actual: Vec<usize> = q
                    .answers
                    .iter()
                    .take(scores.len())
                    .enumerate()
                    .filter_map(|(i, a)| a.correct.then_some(i))
                    .collect();
                let expected: Vec<usize> =
                    serde_json::from_value(case["expected"].clone()).unwrap();
                assert_eq!(actual, expected);
                if kind == "calculatedmulti" {
                    assert_eq!(q.variables[0].values, vec![2.5, 2.5, 4.]);
                } else {
                    assert!(q.variables.is_empty());
                }
            }
        }
    }
}
