use crate::{
    config::EvalConfig,
    eval::{complete_image, image_data_url},
};
use reqwest::Client;
use serde::Serialize;
use std::{cmp, error::Error, fs, path::Path};

#[derive(Debug, Serialize)]
struct OcrMetrics {
    source_chars: usize,
    output_chars: usize,
    edit_distance: usize,
    lcs_chars: usize,
    cer: f64,
    char_recall: f64,
    char_precision: f64,
}

pub async fn run_ocr(args: &[String]) -> Result<(), Box<dyn Error>> {
    let image_path = required_arg(args, "--image")?;
    let source_path = required_arg(args, "--source")?;
    let prompt_path = required_arg(args, "--prompt")?;
    let out_path = required_arg(args, "--out")?;
    let max_tokens: u32 = required_arg(args, "--max-tokens")?.parse()?;

    let config = EvalConfig::from_env()?;
    let prompt = fs::read_to_string(prompt_path)?;
    let source = fs::read_to_string(source_path)?;
    let image_url = image_data_url(image_path)?;
    let client = Client::new();
    let transcript = complete_image(&client, &config, &image_url, &prompt, max_tokens).await?;

    if let Some(parent) = Path::new(out_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(out_path, &transcript)?;

    let metrics = score_ocr(&source, &transcript);
    println!("{}", serde_json::to_string(&metrics)?);
    Ok(())
}

fn score_ocr(source: &str, output: &str) -> OcrMetrics {
    let source_chars: Vec<char> = normalize_text(source).chars().collect();
    let output_chars: Vec<char> = normalize_text(output).chars().collect();
    let edit_distance = levenshtein(&source_chars, &output_chars);
    let lcs_chars = lcs_len(&source_chars, &output_chars);
    let cer = ratio(edit_distance, source_chars.len());
    let char_recall = ratio(lcs_chars, source_chars.len());
    let char_precision = ratio(lcs_chars, output_chars.len());
    OcrMetrics {
        source_chars: source_chars.len(),
        output_chars: output_chars.len(),
        edit_distance,
        lcs_chars,
        cer,
        char_recall,
        char_precision,
    }
}

fn normalize_text(text: &str) -> String {
    text.replace("\r\n", "\n").trim().to_string()
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn levenshtein(left: &[char], right: &[char]) -> usize {
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    let mut current = vec![0; right.len() + 1];
    for (i, left_char) in left.iter().enumerate() {
        current[0] = i + 1;
        for (j, right_char) in right.iter().enumerate() {
            let cost = usize::from(left_char != right_char);
            current[j + 1] = cmp::min(
                cmp::min(current[j] + 1, previous[j + 1] + 1),
                previous[j] + cost,
            );
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right.len()]
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

fn required_arg<'a>(args: &'a [String], key: &str) -> Result<&'a str, Box<dyn Error>> {
    arg_value(args, key).ok_or_else(|| format!("missing required {key}").into())
}

fn arg_value<'a>(args: &'a [String], key: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|window| window[0] == key)
        .map(|window| window[1].as_str())
}
