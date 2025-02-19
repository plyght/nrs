use std::env;
use crate::MyError;
use async_openai::Client;
use async_openai::types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs, Role};

/// Blocking function to get a summary via OpenAI.
pub fn openai_summarize_blocking(content: String) -> Result<String, MyError> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let key = env::var("OPENAI_API_KEY").map_err(|_| "Please set OPENAI_API_KEY!")?;
        let cli = Client::new().with_api_key(key);
        let req = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o-latest")
            .messages(vec![
                ChatCompletionRequestMessage {
                    role: Role::System,
                    content: "You are a helpful assistant that summarizes notes.".to_string(),
                    name: None,
                },
                ChatCompletionRequestMessage {
                    role: Role::User,
                    content: format!("Summarize:\n\n{}", content),
                    name: None,
                },
            ])
            .build()?;
        let resp = cli.chat().create(req).await?;
        let txt = resp.choices.first()
            .map(|c| c.message.content.clone())
            .unwrap_or_else(|| "No summary received.".to_string());
        Ok(txt)
    })
}

/// Blocking function to extract keywords via OpenAI.
pub fn openai_keywords_blocking(content: String) -> Result<String, MyError> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let key = env::var("OPENAI_API_KEY").map_err(|_| "Please set OPENAI_API_KEY!")?;
        let cli = Client::new().with_api_key(key);
        let req = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o-latest")
            .messages(vec![
                ChatCompletionRequestMessage {
                    role: Role::System,
                    content: "You are a helpful assistant that extracts keywords.".to_string(),
                    name: None,
                },
                ChatCompletionRequestMessage {
                    role: Role::User,
                    content: format!("Extract keywords:\n\n{}", content),
                    name: None,
                },
            ])
            .build()?;
        let resp = cli.chat().create(req).await?;
        let txt = resp.choices.first()
            .map(|c| c.message.content.clone())
            .unwrap_or_else(|| "No keywords received.".to_string());
        Ok(txt)
    })
}