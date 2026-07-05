use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use image::{GrayImage, Luma};
use reqwest::Client;
use rusttype::{Font, Scale, point};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{env, error::Error, fs, path::Path, process::Command};

const DEFAULT_MODEL: &str = "google-antigravity/gemini-3.5-flash";
const DEFAULT_BASE_URL: &str = "http://127.0.0.1:4000/v1";
const DEFAULT_TEXT: &str = "【技术报告：多模态光学上下文压缩 (Optical Context Compression)】\n\nOptical Context Compression (OCC) 是一种利用视觉大模型（VLM）处理超长上下文的新型范式。在本测试中，我们将严格评估 Zpix 像素字体在 CJK（中日韩）及 Latin 混合排版下的表现。\n\n1. 背景 (Background)\n随着大语言模型（LLM）的 Context Window 越来越大，Token 计费成为了工程瓶颈。传统系统通过截断或 LLM Summarization 来进行 Memory Compaction。而 OCC 方案直接将对话历史序列化为极高密度的 1-bit Bitmap PNG，利用 Vision API 极低的图像切片价格实现“账单降维打击”。\n\n2. 挑战 (Challenges)\n原生 OMP 系统仅搭载了极小的 ASCII X11 点阵字体。当遇到中文字符时，TypeScript 层的 normalize() 会发生静默破坏，将所有 CJK 字符暴力替换为 `?`。这种做法虽然保持了原有的字符物理数量，却将 Semantic Entropy（语义信息熵）瞬间降为 0。最终导致用户支付了昂贵的 Vision Token 费用，却给大模型发送了满屏无法解析的 Noise 数据。\n\n3. 解决方案 (Proposed Solution)\n采用系统级 Dynamic Glyph Atlas（动态字形图集）架构。当前演示环境使用 Zpix 纯正像素字体，配以 12px 的严格 Grid 渲染，并在光栅化（Rasterization）阶段加入绝对阈值滤镜（Thresholding: 若 coverage v > 0.3 则强制涂成 Pure Black）。\n\n4. 理论优势 (Theoretical Advantages)\n实验证明，同等语义下，中文凭借其方块字的表意特性，占据的总物理像素面积远小于由字母拼接的英文单词。修复 CJK 渲染不仅解决了非英语区用户的可用性灾难，更为真正的 OCC 光学压缩开辟了一条【高语义密度、低帧数占用】的黄金路径！\n\n结论 (Conclusion)\n白底黑字（Black Ink on White Canvas）在各项公开的 SQuAD OCR 评测中表现最佳，能够完美匹配现代多模态大模型底层卷积核提取高频边缘特征的偏好。";

#[derive(Debug, Deserialize)]
struct Probe {
    id: String,
    question: String,
    #[serde(default)]
    expect_contains: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ProbeResult {
    id: String,
    passed: bool,
    missing: Vec<String>,
    answer: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None | Some("render") => {
            let output = args.get(1).map(String::as_str).unwrap_or("output_long.png");
            render_demo(output)?;
        }
        Some("eval") => run_eval(&args[1..]).await?,
        Some("help") | Some("--help") | Some("-h") => print_help(),
        Some(other) => {
            return Err(format!("unknown command `{other}`; use `render` or `eval`").into());
        }
    }
    Ok(())
}

fn print_help() {
    println!("cjk-raster-playground");
    println!();
    println!("Commands:");
    println!("  cargo run -- render [output.png]");
    println!("  cargo run -- eval --image output_long.png --probes probes/demo.json");
    println!();
    println!("Env:");
    println!("  OMP_GATEWAY_BASE_URL or OPENAI_BASE_URL   default: {DEFAULT_BASE_URL}");
    println!("  OMP_GATEWAY_API_KEY  or OPENAI_API_KEY    default: `omp auth-gateway token`");
    println!("  OMP_GATEWAY_MODEL    or OPENAI_MODEL      default: {DEFAULT_MODEL}");
}

fn render_demo(output: &str) -> Result<(), Box<dyn Error>> {
    let font_data = fs::read("zpix.ttf")?;
    let font = Font::try_from_bytes(&font_data).ok_or("could not parse zpix.ttf")?;
    let scale = Scale::uniform(12.0);
    let v_metrics = font.v_metrics(scale);
    let line_height = (v_metrics.ascent - v_metrics.descent).ceil() + 6.0;
    let max_width = 750.0;
    let margin = 15.0;
    let mut current_x = margin;
    let mut current_y = v_metrics.ascent + margin;
    let mut glyphs = Vec::new();

    for c in DEFAULT_TEXT.chars() {
        if c == '\n' {
            current_x = margin;
            current_y += line_height;
            continue;
        }

        let scaled_glyph = font.glyph(c).scaled(scale);
        let advance_width = scaled_glyph.h_metrics().advance_width;
        if current_x + advance_width > max_width {
            current_x = margin;
            current_y += line_height;
        }

        glyphs.push(scaled_glyph.positioned(point(current_x, current_y)));
        current_x += advance_width;
    }

    let img_width = (max_width + margin) as u32;
    let img_height = (current_y + v_metrics.descent).ceil() as u32 + margin as u32;
    let mut image = GrayImage::from_pixel(img_width, img_height, Luma([255]));

    for glyph in glyphs {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, coverage| {
                let px = x as i32 + bb.min.x;
                let py = y as i32 + bb.min.y;
                if px >= 0
                    && px < image.width() as i32
                    && py >= 0
                    && py < image.height() as i32
                    && coverage > 0.3
                {
                    image.put_pixel(px as u32, py as u32, Luma([0]));
                }
            });
        }
    }

    image.save(output)?;
    println!("rendered {output} ({img_width}x{img_height})");
    Ok(())
}

async fn run_eval(args: &[String]) -> Result<(), Box<dyn Error>> {
    let image_path = arg_value(args, "--image").unwrap_or("output_long.png");
    let probes_path = arg_value(args, "--probes").unwrap_or("probes/demo.json");
    let model = env::var("OMP_GATEWAY_MODEL")
        .or_else(|_| env::var("OPENAI_MODEL"))
        .unwrap_or_else(|_| DEFAULT_MODEL.to_string());
    let base_url = env::var("OMP_GATEWAY_BASE_URL")
        .or_else(|_| env::var("OPENAI_BASE_URL"))
        .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
    let api_key = resolve_api_key(&base_url)?;
    let probes: Vec<Probe> = serde_json::from_str(&fs::read_to_string(probes_path)?)?;
    let image_url = image_data_url(image_path)?;
    let client = Client::new();

    let mut passed = 0usize;
    let mut results = Vec::with_capacity(probes.len());
    for probe in probes {
        let answer = ask_image(
            &client,
            &base_url,
            &api_key,
            &model,
            &image_url,
            &probe.question,
        )
        .await?;
        let missing: Vec<String> = probe
            .expect_contains
            .iter()
            .filter(|expected| !answer.contains(expected.as_str()))
            .cloned()
            .collect();
        let ok = missing.is_empty();
        if ok {
            passed += 1;
        }
        let result = ProbeResult {
            id: probe.id,
            passed: ok,
            missing,
            answer,
        };
        println!("{}", serde_json::to_string(&result)?);
        results.push(result);
    }

    let total = results.len();
    let rate = if total == 0 {
        0.0
    } else {
        passed as f64 / total as f64 * 100.0
    };
    eprintln!("score: {passed}/{total} ({rate:.1}%)");
    if passed != total {
        return Err("one or more probes failed".into());
    }
    Ok(())
}

fn arg_value<'a>(args: &'a [String], key: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|window| window[0] == key)
        .map(|window| window[1].as_str())
}

fn resolve_api_key(base_url: &str) -> Result<String, Box<dyn Error>> {
    if let Ok(value) = env::var("OMP_GATEWAY_API_KEY") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    let is_default_gateway = base_url.trim_end_matches('/') == DEFAULT_BASE_URL;
    if !is_default_gateway {
        if let Ok(value) = env::var("OPENAI_API_KEY") {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }

    let output = Command::new("omp")
        .args(["auth-gateway", "token"])
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "`omp auth-gateway token` failed with status {}",
            output.status
        )
        .into());
    }
    let token = String::from_utf8(output.stdout)?.trim().to_string();
    if token.is_empty() {
        return Err("`omp auth-gateway token` returned empty token".into());
    }
    Ok(token)
}

fn image_data_url(path: &str) -> Result<String, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let mime = match Path::new(path).extension().and_then(|ext| ext.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        _ => "image/png",
    };
    Ok(format!("data:{mime};base64,{}", BASE64.encode(bytes)))
}

async fn ask_image(
    client: &Client,
    base_url: &str,
    api_key: &str,
    model: &str,
    image_url: &str,
    question: &str,
) -> Result<String, Box<dyn Error>> {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let body = json!({
        "model": model,
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

    let mut request = client.post(url).json(&body);
    if !api_key.is_empty() {
        request = request.bearer_auth(api_key);
    }
    let response = request.send().await?;
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
