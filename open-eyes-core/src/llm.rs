use crate::config::LlmConfig;
use crate::error::OpenEyesError;
use crate::models::{
    ChartConfig, ChartType, ColumnSchema, SqlGenerationResponse, SummarizationResponse, TableInfo,
    TableSelectionResponse,
};

/// LLM client using OpenAI-compatible chat completions API.
pub struct LlmClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
}

impl LlmClient {
    pub fn new(config: &LlmConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: config.base_url.trim_end_matches('/').to_string(),
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            max_tokens: config.max_tokens,
            temperature: config.temperature,
        }
    }

    /// Phase 1: Select relevant tables from the full catalog.
    pub async fn select_tables(
        &self,
        question: &str,
        tables: &[TableInfo],
        language: &str,
    ) -> Result<Vec<String>, OpenEyesError> {
        let table_list: String = tables
            .iter()
            .map(|t| {
                format!(
                    "- {}: {} ({}rows, columns: {})",
                    t.table_name,
                    t.dataset_title,
                    t.row_count,
                    t.column_names.join(", ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let system = format!(
            r#"You are a data analyst assistant. Given a user question and a list of available data tables, select the 3-5 most relevant tables.
Respond ONLY with valid JSON: {{"tables": ["table_name1", "table_name2", ...]}}
The user's UI language is {language}, but always use the exact table names as listed."#
        );

        let user_msg = format!("Question: {question}\n\nAvailable tables:\n{table_list}");

        let response = self.chat(&system, &user_msg).await?;
        let parsed: TableSelectionResponse = serde_json::from_str(&response).map_err(|e| {
            OpenEyesError::Llm(format!(
                "Failed to parse table selection: {e}\nResponse: {response}"
            ))
        })?;
        Ok(parsed.tables)
    }

    /// Phase 2: Generate DuckDB SQL from the question and table schemas.
    pub async fn generate_sql(
        &self,
        question: &str,
        schemas: &[(String, Vec<ColumnSchema>)],
        language: &str,
    ) -> Result<String, OpenEyesError> {
        let schema_text: String = schemas
            .iter()
            .map(|(name, cols)| {
                let cols_str: String = cols
                    .iter()
                    .map(|c| format!("  {} {}", c.name, c.data_type))
                    .collect::<Vec<_>>()
                    .join(",\n");
                format!("TABLE {name}:\n{cols_str}")
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        let system = format!(
            r#"You are a DuckDB SQL expert. Generate a read-only SELECT query to answer the user's question.
Rules:
- Only SELECT statements (no INSERT, UPDATE, DELETE, DROP, CREATE)
- Always add LIMIT 1000
- Use ILIKE for text matching (German text is common)
- Use DuckDB-compatible syntax
- The user's UI language is {language}
Respond ONLY with valid JSON: {{"sql": "SELECT ..."}}"#
        );

        let user_msg = format!("Question: {question}\n\nTable schemas:\n{schema_text}");

        let response = self.chat(&system, &user_msg).await?;
        let parsed: SqlGenerationResponse = serde_json::from_str(&response).map_err(|e| {
            OpenEyesError::Llm(format!("Failed to parse SQL: {e}\nResponse: {response}"))
        })?;
        Ok(parsed.sql)
    }

    /// Phase 3: Summarize results and recommend chart type.
    pub async fn summarize(
        &self,
        question: &str,
        sql: &str,
        data: &[serde_json::Value],
        language: &str,
    ) -> Result<(String, ChartType, Option<ChartConfig>), OpenEyesError> {
        let data_preview = if data.len() > 10 {
            let preview = &data[..10];
            format!(
                "{} (showing first 10 of {} rows)",
                serde_json::to_string(preview).unwrap_or_default(),
                data.len()
            )
        } else {
            serde_json::to_string(data).unwrap_or_default()
        };

        let lang_instruction = if language == "de" {
            "Respond in German."
        } else {
            "Respond in English."
        };

        let system = format!(
            r#"You are a data analyst. Summarize the query results and recommend a visualization.
{lang_instruction}
Respond ONLY with valid JSON:
{{
  "summary": "Natural language summary of the data",
  "chart_type": "bar|line|pie|scatter|table|none",
  "chart_config": {{"title": "Chart title", "x_field": "column_name", "y_field": "column_name", "series_field": null}}
}}
Set chart_type to "table" if data is best shown as a table, "none" if no visualization makes sense.
chart_config can be null if chart_type is "none"."#
        );

        let user_msg = format!("Question: {question}\nSQL: {sql}\nResults:\n{data_preview}");

        let response = self.chat(&system, &user_msg).await?;
        let parsed: SummarizationResponse = serde_json::from_str(&response).map_err(|e| {
            OpenEyesError::Llm(format!(
                "Failed to parse summary: {e}\nResponse: {response}"
            ))
        })?;
        Ok((parsed.summary, parsed.chart_type, parsed.chart_config))
    }

    /// Send a chat completion request and extract the assistant's message content.
    async fn chat(&self, system: &str, user_msg: &str) -> Result<String, OpenEyesError> {
        let url = format!("{}/chat/completions", self.base_url);

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user_msg }
            ],
            "max_tokens": self.max_tokens,
            "temperature": self.temperature
        });

        let mut req = self.client.post(&url).json(&body);
        if !self.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(OpenEyesError::Llm(format!(
                "LLM API returned {status}: {text}"
            )));
        }

        let json: serde_json::Value = resp.json().await?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| OpenEyesError::Llm("No content in LLM response".into()))?;

        // Strip markdown code fences if present
        let content = content.trim();
        let content = content
            .strip_prefix("```json")
            .or_else(|| content.strip_prefix("```"))
            .unwrap_or(content);
        let content = content.strip_suffix("```").unwrap_or(content);

        Ok(content.trim().to_string())
    }
}
