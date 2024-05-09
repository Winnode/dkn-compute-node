use colored::Colorize;
use dkn_compute::compute::ollama::use_model_with_prompt;
use ollama_rs::models;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::hash::Hash;
use std::io::Read;

/// A `println!` macro that only prints when the `debug_assertions` flag is set, i.e. it wont print when `--release` is used.
#[allow(unused)]
macro_rules! debug_println {
    ($($arg:tt)*) => (if ::std::cfg!(debug_assertions) { ::std::println!($($arg)*); })
}

/// Shareable format string to print results.
macro_rules! result_format_str {
    () => {
        "{:<8} {:<15} {:<18} {:<18} {:<18} {:<18} {:<18} {:<18} {:<18}"
    };
}

#[inline(always)]
fn print_title() {
    println!(
        result_format_str!(),
        "Prompt".blue(),
        "Model".blue(),
        "Call (ns)".red(),
        "Total (ns)".red(),
        "Prompt (t)".yellow(),
        "Prompt (ns)".yellow(),
        "Result (t)".green(),
        "Result (ns)".green(),
        "TPS".blue(),
    );
}

#[tokio::main]
async fn main() {
    let models = ["orca-mini"]; //, "phi3", "llama3", "openhermes"];
    let preset_prompts =  [
        "Give 3 names of famous scientists, 1 Field Medalist, 1 Turing Award recipient and 1 Nobel laureate. Provide only the names, such as: 1. John Doe, 2. Jane Doe, 3. Foo Bar.",
        "What is the name of the first president of Turkish Republic?",
    ];

    // decide on prompts to be used
    let prompts: Vec<String> = match env::var("JSON_PATH") {
        Ok(path) => {
            println!("Reading tasks from: {}", path);
            let jobs = read_json_file::<Vec<Job>>(path.as_str()).unwrap();
            jobs.into_iter().map(|job| job.prompt).collect()
        }
        Err(_) => {
            println!("Using preset prompts.");
            preset_prompts
                .iter()
                .map(|&prompt| prompt.to_string())
                .collect()
        }
    };

    print_title();
    let mut results = Vec::new();
    let mut num_prompts = HashMap::new();
    for (prompt_num, prompt) in prompts.iter().enumerate() {
        // println!("{}{}: {}", "Prompt #".blue(), prompt_num, prompt);
        for model in models {
            // will loop until it can generate a result with "final data"
            // TODO: waiting for issue https://github.com/pepperoni21/ollama-rs/pull/47
            loop {
                let (generation, duration) = use_model_with_prompt(model, prompt).await;

                if let Some(gen_data) = generation.final_data {
                    let result = BenchmarkResult {
                        prompt_num,
                        model: model.to_string(),
                        api_duration: duration.as_nanos(),
                        total_duration: gen_data.total_duration,
                        prompt_eval_count: gen_data.prompt_eval_count,
                        prompt_eval_duration: gen_data.prompt_eval_duration,
                        eval_count: gen_data.eval_count,
                        eval_duration: gen_data.eval_duration,
                        tokens_per_second: ((gen_data.eval_count as f64)
                            / (gen_data.eval_duration as f64)
                            * 1_000_000_000f64),
                    };

                    println!("{}", result);
                    results.push(result);
                    num_prompts.insert(model, num_prompts.get(model).unwrap_or(&0) + 1);
                    break;
                } else {
                    println!("{}: {}", "Warn".yellow(), "Could not get final data.");
                }
            }
        }
    }

    println!("Average {} for each model:", "tokens per second".yellow());
    // let mut tps = HashMap::new();
    // for result in &results {
    //     tps.insert(
    //         &result.model,
    //         tps.get(&result.model).unwrap_or(&0f64) + result.tokens_per_second,
    //     );
    // }
    // for model in models {
    //     let avg_tps = tps.get(model).unwrap() / num_prompts.get(&model).unwrap() as f64;
    // }
}

/// Reads a JSON file and deserializes it.
fn read_json_file<T: for<'a> Deserialize<'a>>(file_path: &str) -> Result<T, std::io::Error> {
    let mut file = File::open(file_path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let obj = serde_json::from_str(&contents)?;
    Ok(obj)
}

impl std::fmt::Display for BenchmarkResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            result_format_str!(),
            self.prompt_num,
            self.model,
            self.api_duration,
            self.total_duration,
            self.prompt_eval_count,
            self.prompt_eval_duration,
            self.eval_count,
            self.eval_duration,
            self.tokens_per_second
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchmarkResult {
    /// Prompt number
    pub prompt_num: usize,
    /// Model used to generate the result
    pub model: String,
    /// Time spent making the entire API call to Ollama
    pub api_duration: u128,
    /// Time spent evaluating the prompt & generating the response
    pub total_duration: u64,
    /// Number of tokens in the prompt
    pub prompt_eval_count: u16,
    /// Time spent in nanoseconds evaluating the prompt
    pub prompt_eval_duration: u64,
    /// Number of tokens in the response
    pub eval_count: u16,
    /// Time in nanoseconds spent generating the response
    pub eval_duration: u64,
    /// Tokens per second is calculated by `eval_count / eval_duration * 10^9`, see https://github.com/ollama/ollama/blob/main/docs/api.md#response
    pub tokens_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Job {
    purpose: String,
    task_type: String,
    dataset_format: String,
    language: String,
    alignment_preferences: Vec<String>,
    dataset_size: usize,
    id: String,
    private_key: String,
    public_key: String,
    status: String,
    prompt: String,
    subtask_id: String,
}
