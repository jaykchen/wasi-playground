use dotenv::dotenv;
use github_flows::{
    get_octo, listen_to_event, octocrab::models::events::payload::PullRequestEventAction,
    EventPayload,
};
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use std::collections::HashMap;
use std::env;
use std::fs;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let vocab_path = "src/vocab.json";
    let merges_path = "src/merges.txt";

    let bpe = BPE::from_file(vocab_path, merges_path)?;

    let diff_contents =
        fs::read_to_string("src/medium.diff.txt").expect("Failed to read the .diff file");

    let tokens = bpe.tokenize(&diff_contents);

    println!("Tokens: {:?}", tokens.len());
    let head = tokens.iter().take(10).cloned().collect::<Vec<String>>().join(" ");
    let tail = tokens
        .into_iter()
        .rev()
        .take(10)
        .collect::<Vec<String>>()
        .join(" ");

    println!("Head: {:?}\n Tail: {:?}", head, tail);

    Ok(())
}

struct BPE {
    vocab: HashMap<String, u32>,
    merges: Vec<(String, String)>,
}

impl BPE {
    fn new(vocab: HashMap<String, u32>, merges: Vec<(String, String)>) -> Self {
        BPE { vocab, merges }
    }

    fn from_file(vocab_path: &str, merges_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let vocab = fs::read_to_string(vocab_path)?;
        let vocab: HashMap<String, u32> = serde_json::from_str(&vocab)?;

        let merges = fs::read_to_string(merges_path)?;
        let merges: Vec<(String, String)> = merges
            .lines()
            .map(|line| {
                let mut parts = line.split_whitespace();
                (
                    parts.next().unwrap().to_string(),
                    parts.next().unwrap().to_string(),
                )
            })
            .collect();

        Ok(Self::new(vocab, merges))
    }

    fn tokenize(&self, input: &str) -> Vec<String> {
        let mut tokens: Vec<String> = Self::split_on_whitespace(input);

        for (a, b) in &self.merges {
            let mut new_tokens = Vec::new();
            let mut i = 0;

            while i < tokens.len() {
                let current_token = &tokens[i];
                let next_token = if i + 1 < tokens.len() {
                    &tokens[i + 1]
                } else {
                    ""
                };

                if current_token == a && next_token == b {
                    new_tokens.push(format!("{}{}", a, b));
                    i += 2;
                } else {
                    new_tokens.push(current_token.clone());
                    i += 1;
                }
            }

            tokens = new_tokens;
        }

        tokens
    }

    fn split_on_whitespace<'a>(input: &'a str) -> Vec<String> {
        input.split_whitespace().map(|s| s.to_string()).collect()
    }
}
