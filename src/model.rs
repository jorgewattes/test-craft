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
pub fn answer_label(mut index: usize) -> String {
    // MoodTest.py letters_for, L952, source offset 0x7c98:
    // repeated remainder modulo 26, then quotient minus one (a..z, aa..zz, aaa..).
    let mut letters = Vec::new();
    loop {
        letters.push((b'a' + (index % 26) as u8) as char);
        let quotient = index / 26;
        if quotient == 0 {
            break;
        }
        index = quotient - 1;
    }
    letters.into_iter().rev().collect()
}
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
pub enum QuestionStatus {
    Ready,
    Draft,
}
impl Default for QuestionStatus {
    fn default() -> Self {
        Self::Ready
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Question {
    pub id: u64,
    pub title: String,
    pub category: String,
    #[serde(default)]
    pub status: QuestionStatus,
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
    #[serde(default)]
    pub category: String,
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
    pub same_questions: bool,
    #[serde(default)]
    pub plan: Vec<(String, usize)>,
    #[serde(default)]
    pub question_order: Vec<usize>,
    pub title: String,
    pub institution: String,
    pub teacher: String,
    pub date: String,
    pub class_name: String,
    #[serde(default)]
    pub logo: Option<Picture>,
    pub students: Vec<StudentExam>,
}

impl Batch {
    /// Apply a permutation of the generated slots to every student's version.
    /// Values, alternatives and their answer keys travel with the question.
    pub fn apply_question_order(&mut self, order: &[usize]) -> bool {
        let count = self.students.first().map_or(0, |s| s.questions.len());
        if order.len() != count
            || order.iter().copied().collect::<BTreeSet<_>>() != (0..count).collect()
            || self.students.iter().any(|s| s.questions.len() != count)
        {
            return false;
        }
        for student in &mut self.students {
            let original = student.questions.clone();
            student.questions = order.iter().map(|&i| original[i].clone()).collect();
        }
        self.question_order = order.to_vec();
        true
    }

    pub fn move_question(&mut self, from: usize, to: usize) -> bool {
        let count = self.students.first().map_or(0, |s| s.questions.len());
        if from >= count || to >= count || self.students.iter().any(|s| s.questions.len() != count)
        {
            return false;
        }
        if self.question_order.len() != count {
            self.question_order = (0..count).collect();
        }
        self.question_order.swap(from, to);
        for student in &mut self.students {
            student.questions.swap(from, to);
        }
        true
    }
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
    rng: &'a mut StdRng,
    draws: &'a mut BTreeMap<String, f64>,
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
            if self.peek() == '0' {
                let radix = match self.chars.get(self.pos + 1) {
                    Some('x' | 'X') => Some(16),
                    Some('o' | 'O') => Some(8),
                    Some('b' | 'B') => Some(2),
                    _ => None,
                };
                if let Some(radix) = radix {
                    return self.radix_integer(radix);
                }
            }
            let start = self.pos;
            while self.peek().is_ascii_digit() || matches!(self.peek(), '.' | '_') {
                self.pos += 1
            }
            if matches!(self.peek(), 'e' | 'E') {
                self.pos += 1;
                if matches!(self.peek(), '+' | '-') {
                    self.pos += 1
                }
                while self.peek().is_ascii_digit() || self.peek() == '_' {
                    self.pos += 1
                }
            }
            let token = &self.chars[start..self.pos];
            // MoodTest.py avaliar_expressao, ast.parse call at source 0x14a5:
            // Python permits separators only between digits, including exponents.
            if token.iter().enumerate().any(|(i, c)| {
                *c == '_'
                    && (i == 0
                        || !token[i - 1].is_ascii_digit()
                        || !token.get(i + 1).is_some_and(char::is_ascii_digit))
            }) {
                return Err("Separador numérico inválido: use _ entre dois dígitos.".into());
            }
            let number: String = token.iter().filter(|c| **c != '_').collect();
            // A nonzero decimal integer cannot start with 0; floats such as
            // 01.5 / 01e2 and all-zero integers such as 00 remain valid.
            if !number.contains(['.', 'e', 'E'])
                && number.starts_with('0')
                && number.chars().any(|c| c != '0')
            {
                return Err("Remova o zero inicial do número inteiro.".into());
            }
            return number.parse().map_err(|_| "Número inválido.".into());
        }
        if self.peek().is_ascii_alphabetic() || self.peek() == '_' {
            let start = self.pos;
            while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
                self.pos += 1
            }
            let name: String = self.chars[start..self.pos].iter().collect();
            if self.eat('(') {
                let x = self.expr()?;
                if name == "rnd" {
                    if !self.eat(',') {
                        return Err("Use rnd(min,max), com dois limites separados por vírgula. Use ponto nos limites decimais.".into());
                    }
                    let max = self.expr()?;
                    if !self.eat(')') {
                        return Err("Use rnd(min,max), com exatamente dois limites.".into());
                    }
                    if !x.is_finite()
                        || !max.is_finite()
                        || x > max
                        || x.abs() > 1e15
                        || max.abs() > 1e15
                    {
                        return Err("Intervalo inválido em rnd(min,max): mínimo ≤ máximo e limites finitos de até 10^15.".into());
                    }
                    let key = format!("rnd({x},{max})");
                    if let Some(value) = self.draws.get(&key) {
                        return Ok(*value);
                    }
                    let unit = (self.rng.next() >> 11) as f64 / ((1u64 << 53) - 1) as f64;
                    let value = if x == max {
                        x
                    } else {
                        (x * (1. - unit) + max * unit).clamp(x, max)
                    };
                    self.draws.insert(key, value);
                    return Ok(value);
                }
                if !self.eat(')') {
                    return Err("Feche os parênteses da função.".into());
                }
                let value = match name.as_str() {
                    "sqrt" => Ok(x.sqrt()),
                    "abs" => Ok(x.abs()),
                    "sin" => Ok(x.sin()),
                    "cos" => Ok(x.cos()),
                    "tan" => Ok(x.tan()),
                    "log" => Ok(x.ln()),
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
    fn radix_integer(&mut self, radix: u32) -> Result<f64, String> {
        // MoodTest.py ast.parse at source 0x14a5 and integer return at 0xd00:
        // 0x/0o/0b accept one separator after the prefix and between digits.
        self.pos += 2;
        self.eat('_');
        let start = self.pos;
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.pos += 1;
        }
        let token = &self.chars[start..self.pos];
        if token.is_empty()
            || token.iter().enumerate().any(|(i, c)| {
                if *c == '_' {
                    i == 0
                        || token[i - 1].to_digit(radix).is_none()
                        || !token.get(i + 1).is_some_and(|c| c.is_digit(radix))
                } else {
                    c.to_digit(radix).is_none()
                }
            })
        {
            return Err(format!("Número inteiro inválido na base {radix}."));
        }
        let digits: String = token.iter().filter(|c| **c != '_').collect();
        u128::from_str_radix(&digits, radix)
            .map(|value| value as f64)
            .map_err(|_| "Número inteiro fora do limite numérico.".into())
    }
}
#[cfg(test)]
pub fn eval(expr: &str, vars: &BTreeMap<String, f64>) -> Result<f64, String> {
    eval_with_random(
        expr,
        vars,
        &mut StdRng::seed_from_u64(0),
        &mut BTreeMap::new(),
    )
}
fn eval_with_random(
    expr: &str,
    vars: &BTreeMap<String, f64>,
    rng: &mut StdRng,
    draws: &mut BTreeMap<String, f64>,
) -> Result<f64, String> {
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
        // MoodTest.py avaliar_expressao_com_variaveis at source 0x1aa2,
        // str(value) at 0x1bed: dataset values are floats, including integral
        // values. Preserve their decimal token: {x}_0 for x=3 is 3.0_0,
        // not 3_0; {x}.5 must remain invalid rather than becoming 3.5.
        expanded.push_str(&format!("{value:?}"));
        rest = &rest[end + 1..];
    }
    expanded.push_str(rest);
    // MoodTest.py _preprocess_expr, source offset 0x128e (ln rewrite at 0x1370):
    // replace ln( before removing whitespace, so ln (x) is not an alias.
    let normalized = expanded.trim().replace('^', "**").replace("ln(", "log(");
    let compact: Vec<_> = normalized.chars().filter(|c| !c.is_whitespace()).collect();
    let mut parentheses = Vec::new();
    let mut chars = Vec::with_capacity(compact.len());
    for (i, &c) in compact.iter().enumerate() {
        if c == '(' {
            let mut start = i;
            while start > 0
                && (compact[start - 1].is_ascii_alphanumeric() || compact[start - 1] == '_')
            {
                start -= 1;
            }
            parentheses.push(compact[start..i] == ['r', 'n', 'd']);
        } else if c == ')' {
            parentheses.pop();
        }
        chars.push(if c == ',' && parentheses.last() != Some(&true) {
            '.'
        } else {
            c
        });
    }
    // Bound recursive unary and power chains as well as parentheses.
    if chars.len() > 512 {
        return Err("Simplifique a fórmula (até 512 símbolos).".into());
    }
    let mut p = Parser {
        chars,
        pos: 0,
        vars,
        depth: 0,
        rng,
        draws,
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

#[cfg(test)]
pub fn render(text: &str, vars: &BTreeMap<String, f64>) -> Result<String, String> {
    render_with_random(
        text,
        vars,
        &mut StdRng::seed_from_u64(0),
        &mut BTreeMap::new(),
    )
}
fn render_with_random(
    text: &str,
    vars: &BTreeMap<String, f64>,
    rng: &mut StdRng,
    draws: &mut BTreeMap<String, f64>,
) -> Result<String, String> {
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
            out.push_str(&fmt(eval_with_random(e, vars, rng, draws)?))
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
#[derive(Clone, Debug)]
pub struct ValidationExample {
    pub values: BTreeMap<String, f64>,
    seed: u64,
}
pub fn example_preview(q: &Question, example: &ValidationExample) -> Result<Instance, String> {
    render_instance(
        q,
        example.values.clone(),
        &mut StdRng::seed_from_u64(example.seed),
    )
}
fn render_instance(
    q: &Question,
    values: BTreeMap<String, f64>,
    rng: &mut StdRng,
) -> Result<Instance, String> {
    render_instance_inner(q, values, rng, true)
}
fn render_instance_inner(
    q: &Question,
    mut values: BTreeMap<String, f64>,
    rng: &mut StdRng,
    include_pictures: bool,
) -> Result<Instance, String> {
    let mut draws = BTreeMap::new();
    let text = render_with_random(&q.text, &values, rng, &mut draws)
        .map_err(|e| format!("Enunciado: {e}"))?;
    let answers = q
        .answers
        .iter()
        .enumerate()
        .map(|(i, a)| {
            Ok(Answer {
                text: render_with_random(&a.text, &values, rng, &mut draws)
                    .map_err(|e| format!("Alternativa {}: {e}", answer_label(i)))?,
                correct: a.correct,
                pictures: if include_pictures {
                    a.pictures.clone()
                } else {
                    vec![]
                },
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    values.extend(draws);
    Ok(Instance {
        title: q.title.clone(),
        category: q.category.clone(),
        text,
        values,
        pictures: if include_pictures {
            q.pictures.clone()
        } else {
            vec![]
        },
        answers,
    })
}
pub fn duplicate_pairs(p: &Instance) -> Vec<(usize, usize)> {
    let signatures: Vec<_> = p
        .answers
        .iter()
        .map(|a| picture_hash(&a.pictures))
        .collect();
    duplicate_pairs_with_signatures(p, &signatures)
}
fn duplicate_pairs_with_signatures(p: &Instance, signatures: &[u64]) -> Vec<(usize, usize)> {
    let mut seen = BTreeMap::new();
    let mut pairs = Vec::new();
    for (i, a) in p.answers.iter().enumerate() {
        let key = (a.text.trim().to_lowercase(), signatures[i]);
        if let Some(&first) = seen.get(&key) {
            pairs.push((first, i));
        } else {
            seen.insert(key, i);
        }
    }
    pairs
}
fn duplicate_error(q: &Question, p: &Instance, pairs: &[(usize, usize)]) -> Option<String> {
    if pairs.is_empty() {
        return None;
    }
    let details = pairs
        .iter()
        .map(|&(a, b)| {
            format!(
                "Alternativas {} e {} produzem o mesmo resultado: «{}».\n{}: {}\n{}: {}",
                answer_label(a),
                answer_label(b),
                if p.answers[b].text.is_empty() {
                    "(somente imagens)"
                } else {
                    &p.answers[b].text
                },
                answer_label(a),
                q.answers[a].text,
                answer_label(b),
                q.answers[b].text
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Some(format!("{details}\nValores do teste: {}\nA comparação ignora maiúsculas e espaços nas pontas, considera as imagens e usa os resultados com até duas casas decimais. Ajuste as fórmulas ou os intervalos.",
        if p.values.is_empty() { "sem variáveis".into() } else { p.values.iter().map(|(k,v)|format!("{k}={v}")).collect::<Vec<_>>().join(", ") }))
}
pub fn instance(
    q: &Question,
    values: BTreeMap<String, f64>,
    rng: &mut StdRng,
    shuffle: bool,
) -> Result<Instance, String> {
    let mut p = render_instance(q, values, rng)?;
    if let Some(e) = duplicate_error(q, &p, &duplicate_pairs(&p)) {
        return Err(e);
    }
    if shuffle {
        p.answers.shuffle(rng);
    }
    Ok(p)
}
pub fn review_preview(q: &Question, seed: u64) -> Result<Instance, String> {
    let mut rng = StdRng::seed_from_u64(seed);
    let values = sample(q, &mut rng);
    render_instance(q, values, &mut rng)
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
    validate_with_example(q).0
}
pub fn validate_with_example(q: &Question) -> (Result<String, String>, Option<ValidationExample>) {
    let mut example = None;
    let result = validate_inner(q, &mut example);
    (result, example)
}
fn validate_inner(q: &Question, example: &mut Option<ValidationExample>) -> Result<String, String> {
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
    let has_random = std::iter::once(&q.text)
        .chain(q.answers.iter().map(|a| &a.text))
        .any(|text| {
            text.chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>()
                .contains("rnd(")
        });
    let exhaustive = count <= 10000 && !has_random;
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
        let replay = ValidationExample {
            values: values.clone(),
            seed: rng.0,
        };
        let p = render_instance_inner(q, values.clone(), &mut rng, false).map_err(|e| {
            format!(
                "{e}\nValores do teste: {}",
                values
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
        if let Some(error) =
            duplicate_error(q, &p, &duplicate_pairs_with_signatures(&p, &signatures))
        {
            *example = Some(replay);
            return Err(error);
        }
    }
    Ok(if exhaustive {
        format!("Validada em {tests} combinações. Sem erros nas fórmulas.")
    } else {
        "Pré-validada em 256 amostras; sorteios e intervalos extensos não permitem testar todos os valores. Cada prova será validada antes de salvar.".into()
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
    generate_with_mode(
        bank,
        plan,
        names,
        seed,
        title,
        institution,
        teacher,
        date,
        class_name,
        false,
    )
}
pub fn eligible(q: &Question) -> bool {
    q.status == QuestionStatus::Ready && validate(q).is_ok()
}
pub fn generate_with_mode(
    bank: &Bank,
    plan: &[(String, usize)],
    names: &str,
    seed: u64,
    title: &str,
    institution: &str,
    teacher: &str,
    date: &str,
    class_name: &str,
    same_questions: bool,
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
    let mut pools = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for (cat, n) in plan {
        if !seen.insert(cat) {
            return Err("Há categorias repetidas no plano da prova.".into());
        }
        let qs: Vec<_> = bank
            .questions
            .iter()
            .filter(|q| *n > 0 && &q.category == cat && eligible(q))
            .collect();
        if *n > qs.len() {
            return Err(format!(
                "{cat}: há {} questões prontas e válidas, mas foram pedidas {n}.",
                qs.len()
            ));
        }
        pools.insert(cat, qs);
    }
    let mut rng = StdRng::seed_from_u64(seed);
    let mut students = vec![];
    let mut common = BTreeMap::new();
    if same_questions {
        for (cat, n) in plan {
            let mut qs = pools[cat].clone();
            qs.shuffle(&mut rng);
            qs.truncate(*n);
            common.insert(cat, qs);
        }
    }
    for name in names {
        let mut questions = vec![];
        for (cat, n) in plan {
            let mut qs = if same_questions {
                common[cat].clone()
            } else {
                pools[cat].clone()
            };
            if !same_questions {
                qs.shuffle(&mut rng);
            }
            for q in qs.into_iter().take(*n) {
                let values = sample(q, &mut rng);
                questions.push(
                    instance(q, values, &mut rng, !same_questions)
                        .map_err(|e| format!("{}: {e}", q.title))?,
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
        same_questions,
        plan: plan.to_vec(),
        question_order: vec![],
        title: title.into(),
        institution: institution.into(),
        teacher: teacher.into(),
        date: date.into(),
        class_name: class_name.into(),
        logo: None,
        students,
    })
}
/// Stage the whole merge before mutating the user's bank. Category paths are never
/// replaced by the currently selected filter, and colliding source IDs are remapped.
pub fn merge_import(
    bank: &mut Bank,
    incoming: Bank,
    errors: &BTreeSet<u64>,
    include_errors: bool,
) -> Result<usize, String> {
    let questions: Vec<_> = incoming
        .questions
        .into_iter()
        .filter(|q| include_errors || !errors.contains(&q.id))
        .collect();
    if bank.questions.len() + questions.len() > 10000 {
        return Err("A importação excederia o limite de 10.000 questões.".into());
    }
    let mut next = bank.questions.iter().map(|q| q.id).max().unwrap_or(0);
    let mut staged = Vec::with_capacity(questions.len());
    for mut q in questions {
        next = next
            .checked_add(1)
            .ok_or("Não há identificadores disponíveis para importar.")?;
        q.id = next;
        staged.push(q);
    }
    let count = staged.len();
    bank.category_labels.extend(incoming.category_labels);
    bank.questions.extend(staged);
    bank.category_labels = categories(bank);
    Ok(count)
}
/// Move selected subtrees, or delete them with their questions, atomically.
pub fn bulk_categories(
    bank: &mut Bank,
    selected: &BTreeSet<String>,
    destination: Option<&str>,
) -> Result<(), String> {
    let labels = categories(bank);
    if selected.is_empty() || selected.iter().any(|p| !labels.contains(p)) {
        return Err("Selecione categorias existentes.".into());
    }
    let roots: Vec<_> = selected
        .iter()
        .filter(|p| {
            !selected
                .iter()
                .any(|other| *p != other && category_matches(p, other))
        })
        .cloned()
        .collect();
    let mut staged = bank.clone();
    if let Some(destination) = destination {
        if !labels.iter().any(|p| p == destination)
            || roots.iter().any(|p| category_matches(destination, p))
        {
            return Err("Escolha um destino fora das categorias selecionadas.".into());
        }
        for root in &roots {
            let name = root.rsplit('/').next().unwrap();
            rename_category(&mut staged, root, name, destination)?;
        }
    } else {
        staged
            .questions
            .retain(|q| !roots.iter().any(|p| category_matches(&q.category, p)));
        staged.category_labels = labels
            .into_iter()
            .filter(|p| !roots.iter().any(|root| category_matches(p, root)))
            .collect();
    }
    *bank = staged;
    Ok(())
}
pub fn remove_category(bank: &mut Bank, path: &str, destination: &str) -> Result<(), String> {
    let labels = categories(bank);
    if !labels.iter().any(|p| p == path) {
        return Err("Selecione uma categoria existente.".into());
    }
    let occupied = bank
        .questions
        .iter()
        .any(|q| category_matches(&q.category, path));
    if occupied && (!labels.iter().any(|p| p == destination) || category_matches(destination, path))
    {
        return Err("Escolha um destino fora da categoria que será excluída.".into());
    }
    for q in &mut bank.questions {
        if category_matches(&q.category, path) {
            q.category = destination.into();
        }
    }
    bank.category_labels = labels
        .into_iter()
        .filter(|p| !category_matches(p, path))
        .collect();
    Ok(())
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
            status: QuestionStatus::Ready,
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
fn image_sources(html: &str) -> Vec<String> {
    let mut sources = vec![];
    for tag in html
        .split('<')
        .skip(1)
        .filter_map(|s| s.split_once('>').map(|p| p.0))
    {
        let tag = tag.trim();
        if tag.len() < 4
            || !tag
                .get(..3)
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("img"))
            || !tag.as_bytes()[3].is_ascii_whitespace()
        {
            continue;
        }
        let mut attrs = &tag[3..];
        while !attrs.trim().is_empty() {
            attrs = attrs.trim_start();
            let end = attrs
                .find(|c: char| c.is_whitespace() || c == '=')
                .unwrap_or(attrs.len());
            let key = &attrs[..end];
            attrs = attrs[end..].trim_start();
            if !attrs.starts_with('=') {
                if attrs.is_empty() {
                    break;
                }
                continue;
            }
            attrs = attrs[1..].trim_start();
            let (value, rest) = if attrs.starts_with(['\'', '"']) {
                let quote = attrs.as_bytes()[0] as char;
                let value = &attrs[1..];
                match value.find(quote) {
                    Some(end) => (&value[..end], &value[end + 1..]),
                    None => (value, ""),
                }
            } else {
                let end = attrs.find(char::is_whitespace).unwrap_or(attrs.len());
                (&attrs[..end], &attrs[end..])
            };
            if key.eq_ignore_ascii_case("src") {
                sources.push(value.replace("&amp;", "&"));
            }
            attrs = rest;
        }
    }
    sources
}
fn image_path(path: &str) -> String {
    let path = path.strip_prefix("@@PLUGINFILE@@").unwrap_or(path);
    let mut decoded = vec![];
    let mut bytes = path.as_bytes().iter().copied();
    while let Some(b) = bytes.next() {
        if b == b'%' {
            let pair: Vec<_> = bytes.by_ref().take(2).collect();
            if pair.len() == 2 {
                if let Ok(value) = u8::from_str_radix(&String::from_utf8_lossy(&pair), 16) {
                    decoded.push(value);
                    continue;
                }
            }
            decoded.push(b);
            decoded.extend(pair);
        } else {
            decoded.push(b);
        }
    }
    String::from_utf8_lossy(&decoded)
        .trim_start_matches('/')
        .to_owned()
}
fn imported_picture(name: String, encoded: &str, issues: &mut Vec<String>) -> Option<Picture> {
    let result = decode64(encoded).and_then(|data| {
        let picture = Picture {
            name: name.clone(),
            data,
        };
        check_picture(&picture)?;
        Ok(picture)
    });
    match result {
        Ok(picture) => Some(picture),
        Err(error) => {
            issues.push(format!("{name}: {error}"));
            None
        }
    }
}
fn xml_pictures(node: &XmlNode, issues: &mut Vec<String>) -> Vec<Picture> {
    let mut files = vec![];
    node.find("file", &mut files);
    let sources = image_sources(&node.child("text").map(|x| x.text.as_str()).unwrap_or(""));
    let mut pictures = vec![];
    if sources.is_empty() {
        for file in files {
            let name = file
                .attrs
                .get("name")
                .cloned()
                .unwrap_or_else(|| "Imagem".into());
            if let Some(picture) = imported_picture(name, &file.text, issues) {
                pictures.push(picture);
            }
        }
    } else {
        for (i, source) in sources.iter().enumerate() {
            if source.to_ascii_lowercase().starts_with("data:image/") {
                match source
                    .split_once(',')
                    .filter(|(meta, _)| meta.to_ascii_lowercase().ends_with(";base64"))
                {
                    Some((_, encoded)) => {
                        if let Some(picture) = imported_picture(
                            format!("Imagem incorporada {}", i + 1),
                            encoded,
                            issues,
                        ) {
                            pictures.push(picture);
                        }
                    }
                    None => {
                        issues.push("Imagem incorporada sem base64 válido. Use PNG/JPG.".into())
                    }
                }
                continue;
            }
            let path = image_path(source);
            let file = files
                .iter()
                .find(|file| {
                    let name = file.attrs.get("name").map(String::as_str).unwrap_or("");
                    let dir = file.attrs.get("path").map(String::as_str).unwrap_or("/");
                    image_path(&format!("{}/{name}", dir.trim_end_matches('/'))) == path
                })
                .or_else(|| {
                    let matches: Vec<_> = files
                        .iter()
                        .filter(|file| {
                            file.attrs
                                .get("name")
                                .is_some_and(|name| image_path(name) == path)
                        })
                        .collect();
                    if matches.len() == 1 {
                        Some(matches[0])
                    } else {
                        None
                    }
                });
            if let Some(file) = file {
                let name = file
                    .attrs
                    .get("name")
                    .cloned()
                    .unwrap_or_else(|| "Imagem".into());
                if let Some(picture) = imported_picture(name, &file.text, issues) {
                    pictures.push(picture);
                }
            } else {
                issues.push(format!(
                    "Imagem '{source}' não incluída no XML. Anexe a imagem na edição da questão."
                ));
            }
        }
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
    let mut category_labels = vec![];
    let mut questions = vec![];
    for n in nodes {
        let kind = n.attrs.get("type").map(String::as_str).unwrap_or("");
        if kind == "category" {
            let raw = n.nested("category");
            // Moodle's scope is metadata; keep the complete hierarchy after it.
            let path = raw
                .strip_prefix("$course$/")
                .or_else(|| raw.strip_prefix("$system$/"))
                .unwrap_or(&raw)
                .trim_matches('/');
            category = if path.is_empty() {
                "Importadas".into()
            } else {
                path.into()
            };
            category_labels.push(category.clone());
            continue;
        }
        let mut import_issues = vec![];
        if !["calculatedmulti", "multichoice"].contains(&kind) {
            import_issues.push(format!("Tipo Moodle '{kind}' não suportado. Adapte a questão no editor; o XML original foi preservado."));
        }
        let mut pictures = n
            .child("questiontext")
            .map(|node| xml_pictures(node, &mut import_issues))
            .unwrap_or_default();
        // Older Moodle exports store the statement image on the question itself.
        if let Some(encoded) = n
            .child("image_base64")
            .filter(|x| !x.text.trim().is_empty())
        {
            let name = n
                .child("image")
                .map(|x| x.text.clone())
                .unwrap_or_else(|| "Imagem Moodle".into());
            if let Some(picture) = imported_picture(name, &encoded.text, &mut import_issues) {
                pictures.push(picture);
            }
        }
        let mut variables: Vec<Variable> = vec![];
        let mut defs = vec![];
        // MoodTest.py carregar_questoes, source offset 0xc549: only
        // calculatedmulti uses datasets; trim individual values before parsing.
        if kind == "calculatedmulti" {
            if let Some(parent) = n.child("dataset_definitions") {
                defs.extend(
                    parent
                        .children
                        .iter()
                        .filter(|x| x.tag == "dataset_definition"),
                );
            }
        }
        for d in defs {
            let name = d.nested("name");
            // carregar_questoes (0xc549), unnamed guard at source offset 0xcba2.
            if name.is_empty() {
                continue;
            }
            // Original source 0xcbee: follow direct dataset_items/dataset_item/value
            // children, excluding metadata, nested wrappers and extra value nodes.
            let vals: Vec<_> = d
                .child("dataset_items")
                .into_iter()
                .flat_map(|parent| parent.children.iter())
                .filter(|item| item.tag == "dataset_item")
                .filter_map(|item| item.child("value"))
                .collect();
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
                    "Dataset {name} sem valores: definição ignorada. Revise os valores no editor."
                ));
                // Original guard 0xcf9b: an empty definition never installs a
                // variable or replaces earlier valid values with a synthetic zero.
                continue;
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
            status: QuestionStatus::Ready,
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
        category_labels,
        questions,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn random_multiplier_is_bounded_shared_and_reproducible() {
        let mut q = examples().questions.remove(0);
        q.variables.clear();
        q.text = "Fator {=rnd(-0.5,1.5)}, produto {=10*rnd(-0.5,1.5)}".into();
        q.answers = vec![
            Answer {
                text: "{=10*rnd(-0.5,1.5)}".into(),
                correct: true,
                pictures: vec![],
            },
            Answer {
                text: "{=10*rnd(-0.5,1.5)+100}".into(),
                correct: false,
                pictures: vec![],
            },
        ];
        let mut factors = BTreeSet::new();
        for seed in 0..64 {
            let p = preview(&q, seed).unwrap();
            assert_eq!(p, preview(&q, seed).unwrap());
            let factor = p.values["rnd(-0.5,1.5)"];
            assert!((-0.5..=1.5).contains(&factor));
            assert!(p.text.ends_with(&format!("produto {}", p.answers[0].text)));
            assert_eq!(p.answers[0].text, fmt(10. * factor));
            factors.insert(factor.to_bits());
        }
        assert!(factors.len() > 50);
        assert!(validate(&q).unwrap().contains("256 amostras"));
        assert_eq!(eval("3*rnd(2,2)", &BTreeMap::new()).unwrap(), 6.);
        assert_eq!(eval("1,5*rnd(sqrt(4),2)", &BTreeMap::new()).unwrap(), 3.);
        assert_eq!(eval("rnd(rnd(2,2),sqrt(4))", &BTreeMap::new()).unwrap(), 2.);
        assert_eq!(
            eval("rnd({x},{x}+1)", &BTreeMap::from([("x".into(), 2.)]))
                .unwrap()
                .floor(),
            2.
        );
        for e in [
            "rnd(2,1)",
            "rnd(1)",
            "rnd()",
            "rnd(1,2,3)",
            "rnd(1,2",
            "rnd(sqrt(-1),2)",
            "rnd(1e999,2)",
        ] {
            assert!(eval(e, &BTreeMap::new()).is_err(), "{e}");
        }
    }
    #[test]
    fn duplicate_diagnostic_replays_exact_values_and_random_factors() {
        let mut q = examples().questions.remove(0);
        q.variables = vec![Variable {
            name: "x".into(),
            min: 1.,
            max: 2.,
            decimals: 0,
            values: vec![],
        }];
        q.text = "x={x}, fator={=rnd(0.8,1.2)}".into();
        q.answers = vec![
            Answer {
                text: "{={x}*rnd(0.8,1.2)}".into(),
                correct: true,
                pictures: vec![],
            },
            Answer {
                text: "{={x}*rnd(0.8,1.2)+0.000001}".into(),
                correct: false,
                pictures: vec![],
            },
            Answer {
                text: "Outro valor".into(),
                correct: false,
                pictures: vec![],
            },
        ];
        let (result, example) = validate_with_example(&q);
        let error = result.unwrap_err();
        let example = example.unwrap();
        let p = example_preview(&q, &example).unwrap();
        assert_eq!(p, example_preview(&q, &example).unwrap());
        assert_eq!(duplicate_pairs(&p), vec![(0, 1)]);
        assert!(error.contains("Alternativas a e b"));
        assert!(error.contains(&q.answers[1].text));
        assert!(error.contains(&p.answers[0].text));
        assert!(error.contains("x=1"));
        assert!(error.contains(&format!("rnd(0.8,1.2)={}", p.values["rnd(0.8,1.2)"])));
        assert!(instance(
            &q,
            example.values.clone(),
            &mut StdRng::seed_from_u64(example.seed),
            false
        )
        .is_err());
        q.answers[1].text = "{={x}*rnd(0.8,1.2)+100}".into();
        let (result, example) = validate_with_example(&q);
        assert!(result.is_ok());
        assert!(example.is_none());
        // A collision in a later combination must replace the normal random preview.
        q.text = "x={x}".into();
        q.answers[0].text = "{={x}}".into();
        q.answers[1].text = "2".into();
        let (result, example) = validate_with_example(&q);
        assert!(result.unwrap_err().contains("x=2"));
        let p = example_preview(&q, &example.unwrap()).unwrap();
        assert_eq!(p.text, "x=2");
        assert_eq!(p.answers[0].text, p.answers[1].text);
    }

    #[test]
    fn placeholder_spelling_matches_original() {
        let mechanics: serde_json::Value =
            serde_json::from_str(include_str!("../mechanics.json")).unwrap();
        for case in mechanics["original"]["placeholder_spelling_cases"]
            .as_array()
            .unwrap()
        {
            let expression = case["expression"].as_str().unwrap();
            let vars: BTreeMap<String, f64> =
                serde_json::from_value(case["variables"].clone()).unwrap();
            let result = eval(expression, &vars);
            if let Some(expected) = case["value"].as_f64().filter(|v| v.abs() <= 1e15) {
                let actual = result.unwrap_or_else(|e| panic!("{case}: {e}"));
                assert!(
                    (actual - expected).abs() <= expected.abs() * 1e-12 + 1e-100,
                    "{case}: {actual}"
                );
                assert_eq!(
                    render(&format!("{{={expression}}}"), &vars).unwrap(),
                    fmt(expected),
                    "{case}"
                );
            } else {
                assert!(result.is_err(), "{case}: {result:?}");
                assert!(
                    render(&format!("{{={expression}}}"), &vars).is_err(),
                    "{case}"
                );
            }
        }
    }
    #[test]
    fn expression_normalization_matches_original() {
        let mechanics: serde_json::Value =
            serde_json::from_str(include_str!("../mechanics.json")).unwrap();
        let vars = BTreeMap::from([("ln".to_owned(), 7.0)]);
        for case in mechanics["original"]["normalization_cases"]
            .as_array()
            .unwrap()
        {
            let expression = case["expression"].as_str().unwrap();
            let result = eval(expression, &vars);
            let rendered = render(&format!("{{={expression}}}"), &vars);
            if let Some(expected) = case["result"].as_f64() {
                assert_eq!(result.unwrap(), expected, "{expression}");
                assert_eq!(rendered.unwrap(), fmt(expected), "{expression}");
            } else {
                assert!(result.is_err(), "{expression}: {result:?}");
                assert!(rendered.is_err(), "{expression}: {rendered:?}");
            }
        }
    }

    use super::*;
    #[test]
    fn moving_questions_preserves_values_keys_and_saved_order_for_both_modes() {
        let bank = examples();
        let plan = vec![("Mecânica".into(), 2), ("Eletricidade".into(), 1)];
        for mode in [false, true] {
            let mut batch =
                generate_with_mode(&bank, &plan, "Ana\nBia", 42, "Prova", "", "", "", "", mode)
                    .unwrap();
            let original = serde_json::to_value(&batch).unwrap();
            assert!(!batch.move_question(0, 3));
            assert_eq!(serde_json::to_value(&batch).unwrap(), original);
            assert!(batch.move_question(2, 1));
            assert!(batch.move_question(1, 0));
            assert_eq!(batch.question_order, vec![2, 0, 1]);
            for (i, student) in batch.students.iter().enumerate() {
                for (position, source) in [2, 0, 1].iter().enumerate() {
                    assert_eq!(
                        serde_json::to_value(&student.questions[position]).unwrap(),
                        original["students"][i]["questions"][*source]
                    );
                }
            }
            let reopened: Batch =
                serde_json::from_value(serde_json::to_value(&batch).unwrap()).unwrap();
            let mut regenerated =
                generate_with_mode(&bank, &plan, "Ana\nBia", 42, "Prova", "", "", "", "", mode)
                    .unwrap();
            assert!(!regenerated.apply_question_order(&[0, 0, 2]));
            assert!(regenerated.apply_question_order(&reopened.question_order));
            assert_eq!(
                serde_json::to_value(&regenerated).unwrap(),
                serde_json::to_value(&reopened).unwrap()
            );
            let legacy: Batch = serde_json::from_value(original).unwrap();
            assert!(legacy.question_order.is_empty());
        }
    }

    #[test]
    fn moodle_images_resolve_paths_inline_data_and_missing_files_individually() {
        fn encode(bytes: &[u8]) -> String {
            let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
            let mut result = String::new();
            for chunk in bytes.chunks(3) {
                let n = ((chunk[0] as u32) << 16)
                    | ((chunk.get(1).copied().unwrap_or(0) as u32) << 8)
                    | chunk.get(2).copied().unwrap_or(0) as u32;
                result.push(alphabet[((n >> 18) & 63) as usize] as char);
                result.push(alphabet[((n >> 12) & 63) as usize] as char);
                result.push(if chunk.len() > 1 {
                    alphabet[((n >> 6) & 63) as usize] as char
                } else {
                    '='
                });
                result.push(if chunk.len() > 2 {
                    alphabet[(n & 63) as usize] as char
                } else {
                    '='
                });
            }
            result
        }
        let mut png = std::io::Cursor::new(vec![]);
        image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            4,
            3,
            image::Rgb([20, 100, 80]),
        ))
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
        let encoded = encode(png.get_ref());
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("checks/moodle-images");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("images.xml");
        std::fs::write(&path, format!(r#"<quiz><question type="multichoice"><name><text>Imagens Moodle</text></name>
          <questiontext><text><![CDATA[<p>Observe:</p><IMG alt='figura' SRC = '@@PLUGINFILE@@/figuras/diagrama%20um.png'><img src="data:image/png;base64,{encoded}">]]></text>
          <file name="diagrama um.png" path="/figuras/" encoding="base64">{encoded}</file>
          <file name="anexo nao utilizado.png" encoding="base64">{encoded}</file></questiontext>
          <answer fraction="100"><text><![CDATA[Correta <img src="@@PLUGINFILE@@/resposta.png">]]><file name="resposta.png" encoding="base64">{encoded}</file></text></answer>
          <answer fraction="0"><text>Errada</text></answer></question></quiz>"#)).unwrap();
        let bank = import_xml(&path).unwrap();
        let q = &bank.questions[0];
        assert!(q.import_issues.is_empty(), "{:?}", q.import_issues);
        validate(q).unwrap();
        assert_eq!(q.pictures.len(), 2);
        assert_eq!(q.pictures[0].name, "diagrama um.png");
        assert_eq!(q.answers[0].pictures.len(), 1);
        assert!(q
            .pictures
            .iter()
            .chain(&q.answers[0].pictures)
            .all(|p| image::load_from_memory(&p.data).is_ok()));
        save_bank(&dir.join("bank.json"), &bank).unwrap();
        let reopened = read_bank(&dir.join("bank.json")).unwrap();
        assert_eq!(reopened.questions[0].pictures, q.pictures);
        let batch = generate(
            &reopened,
            &[("Importadas".into(), 1)],
            "Ana\nBia",
            42,
            "Imagens",
            "",
            "",
            "",
            "",
        )
        .unwrap();
        let exported = crate::pdf::export(&batch, &dir).unwrap();
        let pdf = std::fs::read(exported.join("todas_as_provas.pdf")).unwrap();
        assert!(pdf.windows(15).any(|w| w == b"/Subtype /Image"));
        let xml = std::fs::read_to_string(&path).unwrap().replace(
            "<p>Observe:</p>",
            "<p>Observe:</p><img src='@@PLUGINFILE@@/ausente.png'>",
        );
        std::fs::write(&path, xml).unwrap();
        let partial = import_xml(&path).unwrap();
        assert_eq!(partial.questions[0].pictures.len(), 2);
        assert_eq!(partial.questions[0].import_issues.len(), 1);
        assert!(partial.questions[0].import_issues[0].contains("ausente.png"));
        assert!(validate(&partial.questions[0]).is_err());
        assert_eq!(
            image_sources("<áéí>texto</áéí><img src='a.png'>"),
            vec!["a.png"]
        );
    }

    #[test]
    fn status_compatibility_and_eligible_generation_modes() {
        let mut bank = examples();
        let mut legacy = serde_json::to_value(&bank.questions[0]).unwrap();
        legacy.as_object_mut().unwrap().remove("status");
        assert_eq!(
            serde_json::from_value::<Question>(legacy).unwrap().status,
            QuestionStatus::Ready
        );
        bank.questions[0].status = QuestionStatus::Draft;
        bank.questions[1].text = "{=1/0}".into();
        assert!(!eligible(&bank.questions[0]));
        assert!(!eligible(&bank.questions[1]));
        let plan = vec![("Mecânica".into(), 1)];
        let batch = generate_with_mode(&bank, &plan, "Ana\nBia", 42, "Prova", "", "", "", "", true)
            .unwrap();
        assert!(batch.same_questions);
        assert!(batch
            .students
            .iter()
            .all(|s| s.questions[0].title == bank.questions[2].title));
        assert!(generate_with_mode(
            &bank,
            &[("Mecânica".into(), 2)],
            "Ana",
            42,
            "Prova",
            "",
            "",
            "",
            "",
            true
        )
        .err()
        .unwrap()
        .contains("prontas e válidas"));
        let restored: Bank = serde_json::from_slice(&serde_json::to_vec(&bank).unwrap()).unwrap();
        assert_eq!(restored.questions[0].status, QuestionStatus::Draft);
        let bank = examples();
        let names = (0..12)
            .map(|n| format!("Aluno {n}"))
            .collect::<Vec<_>>()
            .join("\n");
        let shared =
            generate_with_mode(&bank, &plan, &names, 25, "Prova", "", "", "", "", true).unwrap();
        assert!(shared
            .students
            .iter()
            .all(|s| s.questions[0].title == shared.students[0].questions[0].title));
        assert!(shared
            .students
            .iter()
            .any(|s| s.questions[0].values != shared.students[0].questions[0].values));
        let independent =
            generate_with_mode(&bank, &plan, &names, 25, "Prova", "", "", "", "", false).unwrap();
        assert!(independent
            .students
            .iter()
            .any(|s| s.questions[0].title != independent.students[0].questions[0].title));
        let repeated =
            generate_with_mode(&bank, &plan, &names, 25, "Prova", "", "", "", "", true).unwrap();
        assert_eq!(
            serde_json::to_value(&shared).unwrap(),
            serde_json::to_value(repeated).unwrap()
        );
    }
    #[test]
    fn bulk_category_subtrees_are_atomic_and_count_nested_selection_once() {
        let mut bank = examples();
        bank.category_labels
            .extend(["Destino".into(), "Mecânica/Sub".into(), "Óptica".into()]);
        let selected = BTreeSet::from(["Mecânica".into(), "Mecânica/Sub".into(), "Óptica".into()]);
        let before = serde_json::to_value(&bank).unwrap();
        assert!(bulk_categories(&mut bank, &selected, Some("Mecânica/Sub")).is_err());
        assert_eq!(serde_json::to_value(&bank).unwrap(), before);
        bulk_categories(&mut bank, &selected, Some("Destino")).unwrap();
        assert!(categories(&bank).contains(&"Destino/Mecânica/Sub".into()));
        assert_eq!(bank.questions[0].category, "Destino/Mecânica");
        let count = bank.questions.len();
        bulk_categories(
            &mut bank,
            &BTreeSet::from(["Destino/Mecânica".into(), "Destino/Mecânica/Sub".into()]),
            None,
        )
        .unwrap();
        assert_eq!(bank.questions.len(), count - 3);
        assert!(!categories(&bank)
            .iter()
            .any(|p| category_matches(p, "Destino/Mecânica")));
        let mut bank = examples();
        bank.category_labels
            .extend(["Destino".into(), "Destino/Óptica".into(), "Óptica".into()]);
        let before = serde_json::to_value(&bank).unwrap();
        assert!(bulk_categories(
            &mut bank,
            &BTreeSet::from(["Mecânica".into(), "Óptica".into()]),
            Some("Destino")
        )
        .is_err());
        assert_eq!(serde_json::to_value(bank).unwrap(), before);
    }
    #[test]
    fn deleting_category_preserves_questions_and_requires_external_destination() {
        let mut bank = examples();
        bank.questions.truncate(2);
        bank.questions[0].category = "Origem/Sub".into();
        bank.questions[1].category = "Destino".into();
        bank.category_labels = vec!["Origem/Vazia".into()];
        let before = serde_json::to_vec(&bank).unwrap();
        assert!(remove_category(&mut bank, "Origem", "Origem/Sub").is_err());
        assert!(remove_category(&mut bank, "Origem", "").is_err());
        assert_eq!(serde_json::to_vec(&bank).unwrap(), before);
        remove_category(&mut bank, "Origem", "Destino").unwrap();
        assert_eq!(bank.questions.len(), 2);
        assert!(bank.questions.iter().all(|q| q.category == "Destino"));
        assert_eq!(categories(&bank), vec!["Destino"]);
    }
    #[test]
    fn mixed_category_export_import_preserves_paths_and_remaps_ids() {
        let mut bank = examples();
        bank.questions.truncate(3);
        bank.questions[0].category = "Ciências/Física".into();
        bank.questions[1].category = "Ciências/Química".into();
        bank.questions[2].category = "Revisão".into();
        bank.questions[2].text.clear();
        let ids = bank.questions.iter().map(|q| q.id).collect();
        let selection = selected_bank(&bank, &ids);
        let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("checks/mixed-category-import");
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("selection.json");
        save_bank(&path, &selection).unwrap();
        let incoming = read_bank(&path).unwrap();
        let errors: BTreeSet<_> = incoming
            .questions
            .iter()
            .filter(|q| validate(q).is_err())
            .map(|q| q.id)
            .collect();
        assert_eq!(errors.len(), 1);
        let previous = serde_json::to_vec(&bank).unwrap();
        let mut valid = bank.clone();
        assert_eq!(
            merge_import(&mut valid, incoming.clone(), &errors, false).unwrap(),
            2
        );
        assert_eq!(valid.questions[3].category, "Ciências/Física");
        assert_eq!(valid.questions[4].category, "Ciências/Química");
        assert_eq!(
            valid
                .questions
                .iter()
                .map(|q| q.id)
                .collect::<BTreeSet<_>>()
                .len(),
            5
        );
        assert_eq!(serde_json::to_vec(&bank).unwrap(), previous);
        assert_eq!(merge_import(&mut bank, incoming, &errors, true).unwrap(), 3);
        assert!(validate(bank.questions.last().unwrap()).is_err());
        let generated = generate(
            &valid,
            &[
                ("Ciências/Física".into(), 1),
                ("Ciências/Química".into(), 1),
            ],
            "Aluno",
            17,
            "Prova",
            "",
            "",
            "",
            "",
        )
        .unwrap();
        assert_eq!(
            generated.students[0]
                .questions
                .iter()
                .map(|q| q.category.as_str())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["Ciências/Física", "Ciências/Química"])
        );
    }
    #[test]
    fn moodle_category_markers_keep_full_hierarchy_and_distinct_siblings() {
        let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("checks/mixed-category-import");
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("hierarchy.xml");
        std::fs::write(&path,r#"<quiz>
        <question type="category"><category><text>$course$/Ciências/Física/Revisão</text></category></question>
        <question type="multichoice"><name><text>Física</text></name><questiontext><text>Escolha</text></questiontext><answer fraction="100"><text>Sim</text></answer><answer fraction="0"><text>Não</text></answer></question>
        <question type="category"><category><text>$course$/Ciências/Química/Revisão</text></category></question>
        <question type="multichoice"><name><text>Química</text></name><questiontext><text>Escolha</text></questiontext><answer fraction="100"><text>Sim</text></answer><answer fraction="0"><text>Não</text></answer></question>
        <question type="category"><category><text>$course$/Vazia/Subcategoria</text></category></question>
        </quiz>"#).unwrap();
        let incoming = import_xml(&path).unwrap();
        assert_eq!(incoming.questions[0].category, "Ciências/Física/Revisão");
        assert_eq!(incoming.questions[1].category, "Ciências/Química/Revisão");
        assert!(categories(&incoming).contains(&"Vazia/Subcategoria".into()));
        assert!(incoming.questions.iter().all(|q| validate(q).is_ok()));
    }
    #[test]
    fn import_identifier_exhaustion_is_atomic() {
        let mut bank = examples();
        bank.questions[0].id = u64::MAX;
        let previous = serde_json::to_vec(&bank).unwrap();
        assert!(merge_import(&mut bank, examples(), &BTreeSet::new(), true).is_err());
        assert_eq!(serde_json::to_vec(&bank).unwrap(), previous);
    }
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
    fn radix_literals_match_original() {
        let mechanics: serde_json::Value =
            serde_json::from_str(include_str!("../mechanics.json")).unwrap();
        for case in mechanics["original"]["radix_literal_cases"]
            .as_array()
            .unwrap()
        {
            let expression = case["expression"].as_str().unwrap();
            let result = eval(expression, &BTreeMap::new());
            let rendered = render(&format!("{{={expression}}}"), &BTreeMap::new());
            if case.get("error").is_some() {
                assert!(result.is_err(), "original rejects {expression}: {result:?}");
                assert!(
                    rendered.is_err(),
                    "invalid block {expression}: {rendered:?}"
                );
            } else {
                let expected = case["result"].as_f64().unwrap();
                assert_eq!(
                    result.unwrap_or_else(|e| panic!("{expression}: {e}")),
                    expected
                );
                assert_eq!(rendered.unwrap(), fmt(expected), "block {expression}");
            }
        }
    }
    #[test]
    fn numeric_literal_syntax_matches_original() {
        let mechanics: serde_json::Value =
            serde_json::from_str(include_str!("../mechanics.json")).unwrap();
        for case in mechanics["original"]["numeric_syntax_cases"]
            .as_array()
            .unwrap()
        {
            let expression = case["expression"].as_str().unwrap();
            let result = eval(expression, &BTreeMap::new());
            if case.get("error").is_some() {
                assert!(
                    result.is_err(),
                    "original rejects {expression}; got {result:?}"
                );
            } else {
                assert_eq!(
                    result.unwrap_or_else(|e| panic!("original accepts {expression}: {e}")),
                    case["value"].as_f64().unwrap(),
                    "{expression}"
                );
                // The same parser is used by question and answer rendering.
                assert_eq!(
                    render(&format!("{{={expression}}}"), &BTreeMap::new()).unwrap(),
                    fmt(case["value"].as_f64().unwrap())
                );
            }
        }
        // Separators in numbers must not change names containing underscores.
        let vars = BTreeMap::from([("x_1".into(), 4.)]);
        assert_eq!(eval("x_1+1_000", &vars).unwrap(), 1004.);
    }
    #[test]
    fn answer_labels_match_original_beyond_z() {
        let mechanics: serde_json::Value =
            serde_json::from_str(include_str!("../mechanics.json")).unwrap();
        for case in mechanics["original"]["answer_label_cases"]
            .as_array()
            .unwrap()
        {
            let index = case["index"].as_u64().unwrap() as usize;
            assert_eq!(
                answer_label(index),
                case["expected"].as_str().unwrap(),
                "index {index}"
            );
        }
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
    fn dataset_scope_matches_original_through_generation() {
        let mechanics: serde_json::Value =
            serde_json::from_str(include_str!("../mechanics.json")).unwrap();
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("checks/original-dataset-scope-test");
        std::fs::create_dir_all(&dir).unwrap();
        for case in mechanics["original"]["dataset_scope_cases"]
            .as_array()
            .unwrap()
        {
            let xml = dir.join("datasets.xml");
            std::fs::write(&xml, case["xml"].as_str().unwrap()).unwrap();
            let bank = import_xml(&xml).unwrap();
            let json = dir.join("bank.json");
            save_bank(&json, &bank).unwrap();
            let mut bank = read_bank(&json).unwrap();
            let q = &mut bank.questions[0];
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
            assert_eq!(
                !q.import_issues.is_empty(),
                case["review"].as_bool().unwrap(),
                "{}",
                case["label"]
            );
            if !q.import_issues.is_empty() {
                assert!(validate(q).is_err());
                // The product retains review warnings for malformed imports.
                q.import_issues.clear();
            }
            assert!(validate(q).is_ok(), "{}", case["label"]);
            let expected: Vec<f64> =
                serde_json::from_value(case["expected"][0]["values"].clone()).unwrap();
            for seed in 0..32 {
                let generated = preview(q, seed).unwrap();
                let value = generated.values["x"];
                assert!(
                    expected.contains(&value),
                    "{}: unexpected {value}",
                    case["label"]
                );
                assert_eq!(generated.text, format!("Valor {}", fmt(value)));
                assert_eq!(
                    generated.answers.iter().find(|a| a.correct).unwrap().text,
                    fmt(value)
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
