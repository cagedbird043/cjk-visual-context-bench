use crate::{
    config::EvalConfig,
    eval::{evaluate_image, image_data_url},
    probes::load_probes,
    render::{RenderVariant, render_text},
};
use reqwest::Client;
use serde::Serialize;
use std::{error::Error, fs, io::Write};

#[derive(Debug, Serialize)]
struct MatrixResult<'a> {
    variant: &'a str,
    probe_id: &'a str,
    passed: bool,
    missing: &'a [String],
    answer: &'a str,
}

pub async fn run_matrix(args: &[String]) -> Result<(), Box<dyn Error>> {
    let out_dir = required_arg(args, "--out")?;
    let probes_path = required_arg(args, "--probes")?;
    let text_path = required_arg(args, "--text")?;
    let variants = load_variants(required_arg(args, "--matrix")?)?;
    let text = fs::read_to_string(text_path)?;
    let config = EvalConfig::from_env()?;
    let probes = load_probes(probes_path)?;
    fs::create_dir_all(format!("{out_dir}/variants"))?;

    let mut jsonl = fs::File::create(format!("{out_dir}/results.jsonl"))?;
    let mut csv = fs::File::create(format!("{out_dir}/scores.csv"))?;
    writeln!(csv, "variant,passed,total,rate,width,height,image")?;

    let client = Client::new();
    let mut report = String::from(
        "# CJK raster eval\n\n| variant | score | rate | image |\n|---|---:|---:|---|\n",
    );

    for variant in variants {
        let safe_name = safe_file_name(&variant.name);
        let image_path = format!("{out_dir}/variants/{safe_name}.png");
        let rendered = render_text(&image_path, &text, &variant)?;
        let image_url = image_data_url(&image_path)?;
        let results = evaluate_image(&client, &config, &image_url, &probes).await?;
        let passed = results.iter().filter(|result| result.passed).count();
        let total = results.len();
        let rate = if total == 0 {
            0.0
        } else {
            passed as f64 / total as f64 * 100.0
        };

        for result in &results {
            let record = MatrixResult {
                variant: &variant.name,
                probe_id: &result.id,
                passed: result.passed,
                missing: &result.missing,
                answer: &result.answer,
            };
            writeln!(jsonl, "{}", serde_json::to_string(&record)?)?;
        }

        writeln!(
            csv,
            "{},{},{},{:.1},{},{},{}",
            csv_escape(&variant.name),
            passed,
            total,
            rate,
            rendered.width,
            rendered.height,
            csv_escape(&image_path),
        )?;
        report.push_str(&format!(
            "| {} | {}/{} | {:.1}% | `{}` |\n",
            variant.name, passed, total, rate, image_path
        ));
        println!("{}: {}/{} ({rate:.1}%)", variant.name, passed, total);
    }

    fs::write(format!("{out_dir}/report.md"), report)?;
    println!("wrote {out_dir}/report.md");
    Ok(())
}

fn load_variants(path: &str) -> Result<Vec<RenderVariant>, Box<dyn Error>> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn required_arg<'a>(args: &'a [String], key: &str) -> Result<&'a str, Box<dyn Error>> {
    arg_value(args, key).ok_or_else(|| format!("missing required {key}").into())
}

fn arg_value<'a>(args: &'a [String], key: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|window| window[0] == key)
        .map(|window| window[1].as_str())
}

fn safe_file_name(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}
