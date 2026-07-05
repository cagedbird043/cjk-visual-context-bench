use std::{env, error::Error};

#[derive(Clone, Debug)]
pub struct EvalConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

impl EvalConfig {
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let base_url = required_env("OPENAI_BASE_URL")?;
        let api_key = required_env("OPENAI_API_KEY")?;
        let model = required_env("OPENAI_MODEL")?;
        Ok(Self {
            base_url,
            api_key,
            model,
        })
    }
}

fn required_env(key: &str) -> Result<String, Box<dyn Error>> {
    let value = env::var(key)?.trim().to_string();
    if value.is_empty() {
        return Err(format!("{key} is empty").into());
    }
    Ok(value)
}
