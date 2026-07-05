use serde::{Deserialize, Serialize};
use std::{error::Error, fs};

#[derive(Clone, Debug, Deserialize)]
pub struct Probe {
    pub id: String,
    pub question: String,
    #[serde(default)]
    pub expect_contains: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ProbeResult {
    pub id: String,
    pub passed: bool,
    pub missing: Vec<String>,
    pub answer: String,
}

pub fn load_probes(path: &str) -> Result<Vec<Probe>, Box<dyn Error>> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

pub fn score_answer(probe: &Probe, answer: String) -> ProbeResult {
    let missing: Vec<String> = probe
        .expect_contains
        .iter()
        .filter(|expected| !answer.contains(expected.as_str()))
        .cloned()
        .collect();
    ProbeResult {
        id: probe.id.clone(),
        passed: missing.is_empty(),
        missing,
        answer,
    }
}

pub fn emit_eval_results(results: &[ProbeResult]) -> Result<usize, Box<dyn Error>> {
    let mut passed = 0usize;
    for result in results {
        if result.passed {
            passed += 1;
        }
        println!("{}", serde_json::to_string(result)?);
    }
    let total = results.len();
    let rate = if total == 0 {
        0.0
    } else {
        passed as f64 / total as f64 * 100.0
    };
    eprintln!("score: {passed}/{total} ({rate:.1}%)");
    Ok(passed)
}
