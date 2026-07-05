use crate::{
    config::EvalConfig,
    eval::{complete_image, image_data_url},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{cmp, error::Error, fs, io::Write, path::Path};

#[derive(Debug, Deserialize)]
pub struct QaItem {
    pub id: String,
    pub question: String,
    pub golds: Vec<String>,
    #[serde(default = "default_score")]
    pub score: String,
}

#[derive(Debug, Serialize)]
pub struct QaResult {
    pub id: String,
    pub answer: String,
    pub exact_match: bool,
    pub score: String,
    pub f1: f64,
    pub best_gold: String,
}

pub async fn run_qa(args: &[String]) -> Result<(), Box<dyn Error>> {
    let image_path = required_arg(args, "--image")?;
    let qa_path = required_arg(args, "--qa")?;
    let out_path = required_arg(args, "--out")?;
    let max_tokens: u32 = arg_value(args, "--max-tokens").unwrap_or("160").parse()?;

    let config = EvalConfig::from_env()?;
    let image_url = image_data_url(image_path)?;
    let items: Vec<QaItem> = serde_json::from_str(&fs::read_to_string(qa_path)?)?;
    let client = Client::new();

    if let Some(parent) = Path::new(out_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let mut out = fs::File::create(out_path)?;

    let mut exact = 0usize;
    let mut f1_sum = 0.0;
    for item in &items {
        let prompt = format!(
            "Answer the question using only the image. Reply with the shortest exact answer span. Do not explain.\n\nQuestion: {}",
            item.question
        );
        let answer = complete_image(&client, &config, &image_url, &prompt, max_tokens).await?;
        let result = score_item(item, answer);
        if result.exact_match {
            exact += 1;
        }
        f1_sum += result.f1;
        writeln!(out, "{}", serde_json::to_string(&result)?)?;
        println!("{}", serde_json::to_string(&result)?);
    }

    let total = items.len();
    let em = ratio(exact, total);
    let f1 = if total == 0 {
        0.0
    } else {
        f1_sum / total as f64
    };
    eprintln!("qa: em={em:.4} f1={f1:.4} exact={exact}/{total}");
    Ok(())
}

pub fn score_item(item: &QaItem, answer: String) -> QaResult {
    let mut best_f1 = 0.0;
    let mut best_gold = String::new();
    let exact_mode = item.score == "exact";
    let answer_norm = if exact_mode {
        normalize_exact(&answer)
    } else {
        normalize_answer(&answer)
    };
    let mut exact_match = false;
    for gold in &item.golds {
        let gold_norm = if exact_mode {
            normalize_exact(gold)
        } else {
            normalize_answer(gold)
        };
        if answer_norm == gold_norm {
            exact_match = true;
        }
        let f1 = if exact_mode {
            char_f1(&answer_norm, &gold_norm)
        } else {
            char_f1(&answer_norm, &gold_norm)
        };
        if best_gold.is_empty() || f1 > best_f1 {
            best_f1 = f1;
            best_gold = gold.clone();
        }
    }
    QaResult {
        id: item.id.clone(),
        answer,
        score: item.score.clone(),
        exact_match,
        f1: best_f1,
        best_gold,
    }
}

fn default_score() -> String {
    "semantic".to_string()
}

fn normalize_exact(text: &str) -> String {
    text.trim().to_string()
}

fn normalize_answer(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter_map(|c| match c {
            '\n' | '\t' | ' ' => None,
            '（' => Some('('),
            '）' => Some(')'),
            '，' | '、' => Some(','),
            '。' => Some('.'),
            '：' => Some(':'),
            '“' | '”' => Some('"'),
            '【' => Some('['),
            '】' => Some(']'),
            c if c.is_ascii_punctuation() => None,
            _ => Some(c),
        })
        .collect()
}

fn char_f1(answer: &str, gold: &str) -> f64 {
    if answer.is_empty() || gold.is_empty() {
        return if answer == gold { 1.0 } else { 0.0 };
    }
    let answer_chars: Vec<char> = answer.chars().collect();
    let gold_chars: Vec<char> = gold.chars().collect();
    let overlap = lcs_len(&answer_chars, &gold_chars);
    if overlap == 0 {
        return 0.0;
    }
    let precision = overlap as f64 / answer_chars.len() as f64;
    let recall = overlap as f64 / gold_chars.len() as f64;
    2.0 * precision * recall / (precision + recall)
}

fn lcs_len(left: &[char], right: &[char]) -> usize {
    let mut previous = vec![0; right.len() + 1];
    let mut current = vec![0; right.len() + 1];
    for left_char in left {
        for (j, right_char) in right.iter().enumerate() {
            current[j + 1] = if left_char == right_char {
                previous[j] + 1
            } else {
                cmp::max(current[j], previous[j + 1])
            };
        }
        std::mem::swap(&mut previous, &mut current);
        current.fill(0);
    }
    previous[right.len()]
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn required_arg<'a>(args: &'a [String], key: &str) -> Result<&'a str, Box<dyn Error>> {
    arg_value(args, key).ok_or_else(|| format!("missing required {key}").into())
}

fn arg_value<'a>(args: &'a [String], key: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|window| window[0] == key)
        .map(|window| window[1].as_str())
}
