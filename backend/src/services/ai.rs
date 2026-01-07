use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize)]
struct LMStudioRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct LMStudioResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

pub struct AIService {
    client: Client,
    api_url: String,
    model_name: String,
}

impl AIService {
    pub fn new(api_url: String, model_name: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        AIService {
            client,
            api_url,
            model_name,
        }
    }

    pub async fn process_analytics_query(
        &self,
        query: &str,
        context: Option<&str>,
    ) -> Result<String, String> {
        let system_message = "You are DencapsBI, an advanced AI analytics assistant. \
            You help users analyze data, generate insights, and create visualizations. \
            Provide clear, actionable, and data-driven responses. \
            When appropriate, suggest SQL queries, statistical analyses, or visualization recommendations.";

        let mut messages = vec![Message {
            role: "system".to_string(),
            content: system_message.to_string(),
        }];

        if let Some(ctx) = context {
            messages.push(Message {
                role: "assistant".to_string(),
                content: format!("Context: {}", ctx),
            });
        }

        messages.push(Message {
            role: "user".to_string(),
            content: query.to_string(),
        });

        let request = LMStudioRequest {
            model: self.model_name.clone(),
            messages,
            temperature: 0.7,
            max_tokens: 2000,
        };

        let response = self
            .client
            .post(&format!("{}/v1/chat/completions", self.api_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to LM Studio: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("LM Studio API error ({}): {}", status, error_text));
        }

        let ai_response: LMStudioResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse LM Studio response: {}", e))?;

        ai_response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| "No response from AI model".to_string())
    }

    pub async fn generate_data_insights(
        &self,
        data_summary: &str,
    ) -> Result<String, String> {
        let query = format!(
            "Analyze the following data summary and provide key insights, trends, and recommendations:\n\n{}",
            data_summary
        );

        self.process_analytics_query(&query, None).await
    }

    pub async fn suggest_visualization(
        &self,
        data_description: &str,
    ) -> Result<String, String> {
        let query = format!(
            "Based on the following data description, suggest the most appropriate visualization types and explain why:\n\n{}",
            data_description
        );

        self.process_analytics_query(&query, None).await
    }
}
