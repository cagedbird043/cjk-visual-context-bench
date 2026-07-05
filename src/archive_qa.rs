use crate::{
    config::EvalConfig,
    eval::{complete_images, complete_text, image_data_url},
    qa::{QaItem, score_item},
};
use reqwest::Client;
use std::{error::Error, fs, io::Write, path::Path};

pub async fn run_archive_qa(args: &[String]) -> Result<(), Box<dyn Error>> {
    let qa_path = required_arg(args, "--qa")?;
    let out_path = required_arg(args, "--out")?;
    let max_tokens: u32 = arg_value(args, "--max-tokens").unwrap_or("220").parse()?;
    let mode = if let Some(text_path) = arg_value(args, "--text") {
        ArchiveInput::Text(fs::read_to_string(text_path)?)
    } else if let Some(images_dir) = arg_value(args, "--images-dir") {
        ArchiveInput::Images(load_image_urls(images_dir)?)
    } else {
        return Err("missing --text or --images-dir".into());
    };

    let config = EvalConfig::from_env()?;
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
        let answer = match &mode {
            ArchiveInput::Text(text) => {
                let prompt = text_prompt(text, &item.question);
                complete_text(&client, &config, &prompt, max_tokens).await?
            }
            ArchiveInput::Images(image_urls) => {
                let prompt = image_prompt(&item.question);
                complete_images(&client, &config, image_urls, &prompt, max_tokens).await?
            }
        };
        let result = score_item(item, answer);
        if result.exact_match {
            exact += 1;
        }
        f1_sum += result.f1;
        writeln!(out, "{}", serde_json::to_string(&result)?)?;
        println!("{}", serde_json::to_string(&result)?);
    }

    let total = items.len();
    let f1 = if total == 0 {
        0.0
    } else {
        f1_sum / total as f64
    };
    let em = if total == 0 {
        0.0
    } else {
        exact as f64 / total as f64
    };
    eprintln!("archive-qa: em={em:.4} f1={f1:.4} exact={exact}/{total}");
    Ok(())
}

enum ArchiveInput {
    Text(String),
    Images(Vec<String>),
}

fn text_prompt(archive_text: &str, question: &str) -> String {
    format!(
        "Use only the archive text below. Output only the answer span. No markdown. No explanation. No coordinates. If the archive does not contain the answer, output exactly 不可确定.\n\nArchive:\n{archive_text}\n\nQuestion: {question}"
    )
}

fn image_prompt(question: &str) -> String {
    format!(
        "Use only the archive images. The images are consecutive frames in order. Output only the answer span. No markdown. No explanation. No coordinates. If the archive does not contain the answer, output exactly 不可确定.\n\nQuestion: {question}"
    )
}

fn load_image_urls(images_dir: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut paths = fs::read_dir(images_dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.retain(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("frame-") && name.ends_with(".png"))
    });
    paths.sort();
    if paths.is_empty() {
        return Err(format!("no frame-*.png files under {images_dir}").into());
    }
    paths
        .iter()
        .map(|path| image_data_url(path.to_string_lossy().as_ref()))
        .collect()
}

fn required_arg<'a>(args: &'a [String], key: &str) -> Result<&'a str, Box<dyn Error>> {
    arg_value(args, key).ok_or_else(|| format!("missing required {key}").into())
}

fn arg_value<'a>(args: &'a [String], key: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|window| window[0] == key)
        .map(|window| window[1].as_str())
}
