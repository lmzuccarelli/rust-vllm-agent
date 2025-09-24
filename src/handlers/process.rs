use crate::config::load::Parameters;
use crate::handlers::common::get_error;
use crate::handlers::document::{Document, DocumentformInterface};
use custom_logger as log;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VllmResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    #[serde(rename = "service_tier")]
    pub service_tier: Value,
    #[serde(rename = "system_fingerprint")]
    pub system_fingerprint: Value,
    pub usage: Usage,
    #[serde(rename = "kv_transfer_params")]
    pub kv_transfer_params: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Choice {
    pub index: i64,
    pub text: String,
    pub logprobs: Value,
    #[serde(rename = "finish_reason")]
    pub finish_reason: String,
    #[serde(rename = "stop_reason")]
    pub stop_reason: i64,
    #[serde(rename = "token_ids")]
    pub token_ids: Value,
    #[serde(rename = "prompt_logprobs")]
    pub prompt_logprobs: Value,
    #[serde(rename = "prompt_token_ids")]
    pub prompt_token_ids: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    #[serde(rename = "prompt_tokens")]
    pub prompt_tokens: i64,
    #[serde(rename = "total_tokens")]
    pub total_tokens: i64,
    #[serde(rename = "completion_tokens")]
    pub completion_tokens: i64,
    #[serde(rename = "prompt_tokens_details")]
    pub prompt_tokens_details: Value,
}

pub trait AgentInterface {
    async fn execute(params: Parameters, key: String)
    -> Result<String, Box<dyn std::error::Error>>;
}

pub struct Agent {}

impl AgentInterface for Agent {
    async fn execute(
        params: Parameters,
        key: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let db_path = params.db_path.clone();
        let fd = Document::get_formdata(format!("{}/queue", db_path.clone()), key.clone()).await?;
        log::debug!("[execute] llama agent {:?}", fd);
        let prompt = fd.prompt;
        match params.test {
            true => {
                log::info!("mode: test");
                let data = fs::read(
                    "/home/lzuccarelli/Projects/rust-llama-agent/docs/example-response.json",
                )?;
                let llama: VllmResponse = serde_json::from_slice(&data)?;
                log::debug!("[execute] result from test {:?}", llama);
                Ok("exit =>".to_string())
            }
            false => {
                log::info!("mode: execute");
                let llama_url = format!("{}", params.base_url);
                log::debug!("[execute] url :{}:", llama_url);
                let llama_payload = get_llama_payload(prompt);
                log::debug!("payload {}", llama_payload);
                let client = reqwest::Client::builder()
                    .danger_accept_invalid_certs(true)
                    .build()?;
                let mut headers = HeaderMap::new();
                headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
                headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                let res = client
                    .post(llama_url)
                    .headers(headers)
                    .body(llama_payload)
                    .send()
                    .await;
                match res {
                    Ok(data) => {
                        log::debug!("[execute] waiting for body");
                        let data_result = data.bytes().await;
                        log::debug!("[execute] received body");
                        if data_result.is_ok() {
                            let vllm: VllmResponse = serde_json::from_slice(&data_result.unwrap())?;
                            let vllm_document = vllm.choices[0].text.clone();
                            log::info!("result from llama\n\n {}", vllm_document);
                            Document::save_formdata(db_path, key, vllm_document).await?;
                            Ok("exit => 0".to_string())
                        } else {
                            Err(get_error("[execute] body data error".to_string()))
                        }
                    }
                    Err(e) => Err(get_error(format!("error occured {}", e.to_string()))),
                }
            }
        }
    }
}

fn get_llama_payload(prompt: String) -> String {
    let formatted = prompt
        .split_whitespace()
        .map(|v| v.to_string())
        .collect::<Vec<String>>();
    let data = format!(r#"{{"prompt": "{}"}}"#, formatted.join(" "));
    data
}
