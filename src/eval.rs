use crate::{
    config::EvalConfig,
    probes::{Probe, ProbeResult, emit_eval_results, load_probes, score_answer},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use reqwest::Client;
use serde_json::{Value, json};
use std::{error::Error, fs, path::Path};

pub async fn run_eval(args: &[String]) -> Result<(), Box<dyn Error>> {
    let image_path = required_arg(args, "--image")?;
    let probes_path = required_arg(args, "--probes")?;
    let config = EvalConfig::from_env()?;
    let probes = load_probes(probes_path)?;
    let image_url = image_data_url(image_path)?;
    let client = Client::new();
    let results = evaluate_image(&client, &config, &image_url, &probes).await?;
    let passed = emit_eval_results(&results)?;
    if passed != results.len() {
        return Err("one or more probes failed".into());
    }
    Ok(())
}

pub async fn evaluate_image(
    client: &Client,
    config: &EvalConfig,
    image_url: &str,
    probes: &[Probe],
) -> Result<Vec<ProbeResult>, Box<dyn Error>> {
    let mut results = Vec::with_capacity(probes.len());
    for probe in probes {
        let answer = ask_image(client, config, image_url, &probe.question).await?;
        results.push(score_answer(probe, answer));
    }
    Ok(results)
}

pub fn image_data_url(path: &str) -> Result<String, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let mime = match Path::new(path).extension().and_then(|ext| ext.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        _ => "image/png",
    };
    Ok(format!("data:{mime};base64,{}", BASE64.encode(bytes)))
}

async fn ask_image(
    client: &Client,
    config: &EvalConfig,
    image_url: &str,
    question: &str,
) -> Result<String, Box<dyn Error>> {
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let body = json!({
        "model": config.model,
        "messages": [{
            "role": "user",
            "content": [
                { "type": "text", "text": question },
                { "type": "image_url", "image_url": { "url": image_url } }
            ]
        }],
        "max_tokens": 160,
        "stream": false
    });

    let response = client
        .post(url)
        .bearer_auth(&config.api_key)
        .json(&body)
        .send()
        .await?;
    let status = response.status();
    let text = response.text().await?;
    if !status.is_success() {
        return Err(format!("completion API returned {status}: {text}").into());
    }

    let payload: Value = serde_json::from_str(&text)?;
    let content = payload
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing choices[0].message.content in response: {text}"))?;
    Ok(content.trim().to_string())
}

fn required_arg<'a>(args: &'a [String], key: &str) -> Result<&'a str, Box<dyn Error>> {
    arg_value(args, key).ok_or_else(|| format!("missing required {key}").into())
}

fn arg_value<'a>(args: &'a [String], key: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|window| window[0] == key)
        .map(|window| window[1].as_str())
}
