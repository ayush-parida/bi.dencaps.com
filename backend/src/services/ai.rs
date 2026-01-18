use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::pin::Pin;
use futures::Stream;
use bytes::Bytes;
use crate::config::AIProvider;
use crate::models::StructuredResponse;

// ============================================================================
// OpenAI / LM Studio Compatible Request/Response Structures
// ============================================================================

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
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
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

// ============================================================================
// Custom RAG API Request/Response Structures
// ============================================================================

#[derive(Debug, Serialize)]
struct CustomRAGRequest {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<i32>,
    temperature: f32,
    max_tokens: i32,
}

#[derive(Debug, Deserialize)]
struct CustomRAGResponse {
    #[serde(default)]
    session_id: Option<String>,
    message: RAGMessage,
    #[serde(default)]
    sources: Vec<RAGSource>,
}

#[derive(Debug, Deserialize, Clone)]
struct RAGMessage {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    role: String,
    content: String,
    #[serde(default)]
    sources: Vec<RAGSource>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct RAGSource {
    #[serde(default)]
    content: String,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

// ============================================================================
// AI Service Implementation
// ============================================================================

#[derive(Clone)]
pub struct AIService {
    client: Client,
    api_url: String,
    model_name: String,
    provider: AIProvider,
    api_key: Option<String>,
}

impl AIService {
    pub fn new(api_url: String, model_name: String, provider: AIProvider, api_key: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        AIService {
            client,
            api_url,
            model_name,
            provider,
            api_key,
        }
    }

    /// Get the provider name for logging purposes
    fn provider_name(&self) -> &str {
        match self.provider {
            AIProvider::OpenAI => "OpenAI",
            AIProvider::LMStudio => "LM Studio",
            AIProvider::CustomRAG => "Custom RAG API",
        }
    }

    // ========================================================================
    // OpenAI / LM Studio Methods
    // ========================================================================

    /// Build request for OpenAI-compatible APIs (OpenAI, LM Studio)
    fn build_openai_request(&self, request: &ChatCompletionRequest) -> Result<reqwest::RequestBuilder, String> {
        let url = format!("{}/v1/chat/completions", self.api_url);
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        
        // Add authorization header for OpenAI or if API key is provided
        if let Some(ref api_key) = self.api_key {
            let auth_value = format!("Bearer {}", api_key);
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&auth_value)
                    .map_err(|e| format!("Invalid API key format: {}", e))?
            );
        }
        
        Ok(self.client
            .post(&url)
            .headers(headers)
            .json(request))
    }

    /// Send request to OpenAI-compatible APIs
    async fn send_openai_request(&self, request: ChatCompletionRequest) -> Result<String, String> {
        let req = self.build_openai_request(&request)?;
        
        let response = req
            .send()
            .await
            .map_err(|e| format!("Failed to send request to {}: {}", self.provider_name(), e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("{} API error ({}): {}", self.provider_name(), status, error_text));
        }

        let ai_response: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse {} response: {}", self.provider_name(), e))?;

        ai_response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| "No response from AI model".to_string())
    }

    // ========================================================================
    // Custom RAG API Methods
    // ========================================================================

    /// Build request for Custom RAG API
    fn build_rag_request(&self, request: &CustomRAGRequest) -> Result<reqwest::RequestBuilder, String> {
        let url = format!("{}/api/v1/chat", self.api_url);
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        
        // Add X-API-Key header for Custom RAG
        if let Some(ref api_key) = self.api_key {
            headers.insert(
                HeaderValue::from_static("X-API-Key").to_str()
                    .map(|_| "X-API-Key")
                    .map(|name| reqwest::header::HeaderName::from_static(name))
                    .unwrap_or(reqwest::header::HeaderName::from_static("x-api-key")),
                HeaderValue::from_str(api_key)
                    .map_err(|e| format!("Invalid API key format: {}", e))?
            );
        }
        
        Ok(self.client
            .post(&url)
            .headers(headers)
            .json(request))
    }

    /// Send request to Custom RAG API
    async fn send_rag_request(&self, query: &str, session_id: Option<String>, temperature: f32, max_tokens: i32) -> Result<String, String> {
        let request = CustomRAGRequest {
            message: query.to_string(),
            session_id,
            top_k: Some(5),
            temperature,
            max_tokens,
        };

        let url = format!("{}/api/v1/chat", self.api_url);
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        
        // Add X-API-Key header for Custom RAG
        if let Some(ref api_key) = self.api_key {
            headers.insert(
                reqwest::header::HeaderName::from_static("x-api-key"),
                HeaderValue::from_str(api_key)
                    .map_err(|e| format!("Invalid API key format: {}", e))?
            );
        }

        let response = self.client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to {}: {}", self.provider_name(), e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("{} API error ({}): {}", self.provider_name(), status, error_text));
        }

        let rag_response: CustomRAGResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse {} response: {}", self.provider_name(), e))?;

        Ok(rag_response.message.content)
    }

    /// Send streaming request to Custom RAG API
    /// Returns a stream of bytes from the SSE response
    pub async fn send_rag_stream_request(
        &self,
        query: &str,
        session_id: Option<String>,
        top_k: Option<i32>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>, String> {
        let request = CustomRAGRequest {
            message: query.to_string(),
            session_id,
            top_k: top_k.or(Some(3)),
            temperature: 0.7,
            max_tokens: 2048,
        };

        let url = format!("{}/api/v1/chat/stream", self.api_url);
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Add X-API-Key header for Custom RAG
        if let Some(ref api_key) = self.api_key {
            headers.insert(
                reqwest::header::HeaderName::from_static("x-api-key"),
                HeaderValue::from_str(api_key)
                    .map_err(|e| format!("Invalid API key format: {}", e))?
            );
        }

        println!("Sending streaming request to: {}", url);

        let response = self.client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to send streaming request to {}: {}", self.provider_name(), e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("{} API error ({}): {}", self.provider_name(), status, error_text));
        }

        Ok(Box::pin(response.bytes_stream()))
    }

    /// Stream chat message for Custom RAG API
    /// Combines messages into a query and returns a streaming response
    pub async fn stream_chat_message(
        &self,
        message: &str,
        context: Option<&str>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>, String> {
        // Chart instruction for rendering visual charts
        let chart_instruction = r#"
IMPORTANT: When the user asks for a chart or visualization, you MUST output the data in a JSON code block.

Supported chart types: pie, bar, line, doughnut

Use this EXACT format inside a ```json code block:
{
  "type": "pie",
  "title": "Chart Title",
  "labels": ["Category A", "Category B", "Category C"],
  "data": [100, 200, 300]
}

Examples:

For a PIE CHART showing tax vs take-home:
```json
{"type": "pie", "title": "Income Distribution", "labels": ["Tax Payable", "Take-Home Salary"], "data": [351000, 1649000]}
```

For a BAR CHART showing tax by slab:
```json
{"type": "bar", "title": "Tax by Income Slab", "labels": ["0-2.5L", "2.5L-5L", "5L-10L", "10L-20L"], "data": [0, 12500, 100000, 300000]}
```

For a LINE CHART showing trends:
```json
{"type": "line", "title": "Monthly Trend", "labels": ["Jan", "Feb", "Mar"], "data": [100, 150, 200]}
```

For a DOUGHNUT CHART:
```json
{"type": "doughnut", "title": "Expense Breakdown", "labels": ["Rent", "Food", "Transport"], "data": [500, 200, 100]}
```

ALWAYS use the correct "type" field based on what the user asks for:
- "bar chart" or "bar graph" → type: "bar"
- "pie chart" or "pie graph" → type: "pie"
- "line chart" or "line graph" or "trend" → type: "line"
- "doughnut" or "donut" → type: "doughnut"
- Default to "pie" if no specific type is mentioned
"#;

        // Check if user is asking for a chart
        let message_lower = message.to_lowercase();
        let wants_chart = message_lower.contains("chart") 
            || message_lower.contains("pie") 
            || message_lower.contains("bar")
            || message_lower.contains("line graph")
            || message_lower.contains("visualiz")
            || message_lower.contains("graph")
            || message_lower.contains("doughnut")
            || message_lower.contains("donut");

        // Build the full query with context and chart instructions
        let full_query = match (context, wants_chart) {
            (Some(ctx), true) => format!("{}\n\nContext:\n{}\n\nQuery: {}", chart_instruction, ctx, message),
            (Some(ctx), false) => format!("Context:\n{}\n\nQuery: {}", ctx, message),
            (None, true) => format!("{}\n\nQuery: {}", chart_instruction, message),
            (None, false) => message.to_string(),
        };

        self.send_rag_stream_request(&full_query, None, Some(5)).await
    }

    // ========================================================================
    // Unified Chat Request Method
    // ========================================================================

    /// Send a chat request to the configured AI provider
    async fn send_chat_request(&self, messages: Vec<Message>, temperature: f32, max_tokens: i32) -> Result<String, String> {
        println!("##############################################start");
        println!("Sending chat request to {} with model {}", self.provider_name(), self.model_name);
        // Log request details for debugging
        println!("Request URL: {}", match self.provider {
            AIProvider::OpenAI | AIProvider::LMStudio => format!("{}/v1/chat/completions", self.api_url),
            AIProvider::CustomRAG => format!("{}/api/v1/chat", self.api_url),
        });
        println!("Headers: Content-Type: application/json");
        if let Some(ref api_key) = self.api_key {
            match self.provider {
            AIProvider::OpenAI | AIProvider::LMStudio => println!("Headers: Authorization: Bearer {}...", &api_key[..api_key.len().min(10)]),
            AIProvider::CustomRAG => println!("Headers: X-API-Key: {}...", &api_key[..api_key.len().min(10)]),
            }
        }
        println!("Payload: {:?}", serde_json::json!({
            "model": &self.model_name,
            "messages": &messages,
            "temperature": temperature,
            "max_tokens": max_tokens
        }));
        println!("##############################################end");
        match self.provider {
            AIProvider::OpenAI | AIProvider::LMStudio => {
                let request = ChatCompletionRequest {
                    model: self.model_name.clone(),
                    messages,
                    temperature,
                    max_tokens,
                };
                self.send_openai_request(request).await
            }
            AIProvider::CustomRAG => {
                // For RAG, combine messages into a single query
                // Use the last user message as the primary query
                let query = messages
                    .iter()
                    .filter(|m| m.role == "user")
                    .last()
                    .map(|m| m.content.clone())
                    .unwrap_or_default();
                
                // Include context from system messages if present
                let context: Vec<String> = messages
                    .iter()
                    .filter(|m| m.role == "system" || m.role == "assistant")
                    .map(|m| m.content.clone())
                    .collect();
                
                let full_query = if context.is_empty() {
                    query
                } else {
                    format!("Context:\n{}\n\nQuery: {}", context.join("\n"), query)
                };
                
                self.send_rag_request(&full_query, None, temperature, max_tokens).await
            }
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

        self.send_chat_request(messages, 0.7, 2000).await
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

    pub async fn process_chat_message(
        &self,
        message: &str,
        context: Option<&str>,
    ) -> Result<String, String> {
        let system_message = "You are DencapsBI Chat Assistant, an advanced AI analytics assistant. \
            You help users with data analysis, business intelligence questions, and provide insights. \
            You can discuss data visualization, analytics strategies, SQL queries, and statistical methods. \
            Provide clear, structured, and actionable responses. When providing structured data like lists, \
            tables, or code snippets, use markdown formatting.";

        let mut messages = vec![Message {
            role: "system".to_string(),
            content: system_message.to_string(),
        }];

        if let Some(ctx) = context {
            messages.push(Message {
                role: "system".to_string(),
                content: format!("Previous conversation:\n{}", ctx),
            });
        }

        messages.push(Message {
            role: "user".to_string(),
            content: message.to_string(),
        });

        println!("Sending request to {} API: {}", self.provider_name(), self.api_url);
        self.send_chat_request(messages, 0.7, 2000).await
    }

    pub async fn process_chat_message_structured(
        &self,
        message: &str,
        context: Option<&str>,
    ) -> Result<StructuredResponse, String> {
        let system_message = "You are DencapsBI Chat Assistant, an advanced AI analytics assistant. \
            You respond with structured JSON that can include text, charts, equations, tables, and datasets. \
            \
            Response Format (JSON):\n\
            {\n\
              \"items\": [\n\
                {\"type\": \"text\", \"content\": \"Your explanation here\"},\n\
                {\"type\": \"chart\", \"data\": {\"chart_type\": \"bar|line|pie\", \"title\": \"Chart Title\", \"labels\": [\"A\", \"B\"], \"datasets\": [{\"label\": \"Series\", \"data\": [1, 2]}]}},\n\
                {\"type\": \"equation\", \"latex\": \"E = mc^2\", \"display\": true},\n\
                {\"type\": \"table\", \"data\": {\"headers\": [\"Col1\", \"Col2\"], \"rows\": [[\"val1\", \"val2\"]]}},\n\
                {\"type\": \"dataset\", \"data\": {\"name\": \"Dataset Name\", \"description\": \"Optional\", \"columns\": [{\"name\": \"col\", \"data_type\": \"string\"}], \"rows\": [[\"value\"]]}}\n\
              ]\n\
            }\n\
            \
            Always respond with valid JSON. Use text type for explanations, chart for visualizations, \
            equation for math (LaTeX), table for tabular data, and dataset for structured data with schema.";

        let mut messages = vec![Message {
            role: "system".to_string(),
            content: system_message.to_string(),
        }];

        if let Some(ctx) = context {
            messages.push(Message {
                role: "system".to_string(),
                content: format!("Previous conversation:\n{}", ctx),
            });
        }

        messages.push(Message {
            role: "user".to_string(),
            content: message.to_string(),
        });

        let content = self.send_chat_request(messages, 0.7, 3000).await?;

        // Parse the structured response
        self.parse_and_validate_structured_response(&content)
    }

    fn parse_and_validate_structured_response(
        &self,
        content: &str,
    ) -> Result<StructuredResponse, String> {
        // Try to extract JSON from markdown code blocks if present
        let json_content = if content.contains("```json") {
            content
                .split("```json")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(content)
                .trim()
        } else if content.contains("```") {
            content
                .split("```")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(content)
                .trim()
        } else {
            content.trim()
        };

        // Parse JSON
        let structured_response: StructuredResponse = serde_json::from_str(json_content)
            .map_err(|e| format!("Failed to parse AI response as JSON: {}. Response was: {}", e, json_content))?;

        // Validate the structured response
        structured_response.validate_content()
            .map_err(|e| format!("Invalid structured response: {}", e))?;

        Ok(structured_response)
    }
}
