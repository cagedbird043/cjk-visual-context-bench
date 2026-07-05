mod archive;
mod archive_qa;
mod config;
mod eval;
mod matrix;
mod ocr;
mod probes;
mod qa;
mod render;

use std::{env, error::Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("render") => render::run_render(&args[1..]),
        Some("render-archive") => render::run_render_archive(&args[1..]),
        Some("eval") => eval::run_eval(&args[1..]).await,
        Some("ocr") => ocr::run_ocr(&args[1..]).await,
        Some("qa") => qa::run_qa(&args[1..]).await,
        Some("archive-qa") => archive_qa::run_archive_qa(&args[1..]).await,
        Some("matrix") => matrix::run_matrix(&args[1..]).await,
        Some("export-session") => archive::run_export_session(&args[1..]),
        Some("help") | Some("--help") | Some("-h") | None => {
            print_help();
            Ok(())
        }
        Some(other) => Err(format!(
            "unknown command `{other}`; use `render`, `render-archive`, `eval`, `ocr`, `qa`, `archive-qa`, `matrix`, or `export-session`"
        )
        .into()),
    }
}

fn print_help() {
    println!("cjk-visual-context-bench");
    println!();
    println!("Commands:");
    println!("  cargo run -- render --text <input.txt> --out <output.png> [render knobs]");
    println!(
        "  cargo run -- render-archive --text <input.txt> --out-dir <frames-dir> --frame-size 1568 [--content-width <px>] [--content-height <px>] [render knobs]"
    );
    println!("  cargo run -- eval --image <image.png> --probes <probes.json>");
    println!(
        "  cargo run -- ocr --image <image.png> --source <source.txt> --prompt <prompt.txt> --out <transcript.txt> --max-tokens <n>"
    );
    println!("  cargo run -- qa --image <image.png> --qa <qa.json> --out <results.jsonl>");
    println!(
        "  cargo run -- archive-qa --text <archive.txt>|--images-dir <frames-dir> --qa <qa.json> --out <results.jsonl>"
    );
    println!(
        "  cargo run -- matrix --text <input.txt> --matrix <variants.json> --probes <probes.json> --out <run-dir>"
    );
    println!(
        "  cargo run -- export-session --session <session.jsonl> --compaction-index <n> --out <fixture-dir>"
    );
    println!();
    println!("Required env for eval/matrix:");
    println!("  OPENAI_BASE_URL");
    println!("  OPENAI_API_KEY");
    println!("  OPENAI_MODEL");
    println!();
    println!(
        "Use scripts/*.sh for demo defaults. Core binary keeps paths/models outside compiled code."
    );
}
