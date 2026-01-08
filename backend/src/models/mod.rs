use serde::{Deserialize, Serialize};
use mongodb::bson::{oid::ObjectId, DateTime};
use validator::Validate;

// Custom serialization for UUID as string in MongoDB
mod uuid_as_string {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use uuid::Uuid;

    pub fn serialize<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&uuid.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Uuid::parse_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UserRole {
    Admin,
    ProjectOwner,
    ProjectMember,
    Viewer,
}

impl UserRole {
    pub fn as_str(&self) -> &str {
        match self {
            UserRole::Admin => "admin",
            UserRole::ProjectOwner => "project_owner",
            UserRole::ProjectMember => "project_member",
            UserRole::Viewer => "viewer",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(UserRole::Admin),
            "project_owner" => Some(UserRole::ProjectOwner),
            "project_member" => Some(UserRole::ProjectMember),
            "viewer" => Some(UserRole::Viewer),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(with = "uuid_as_string")]
    pub user_id: uuid::Uuid,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub role: UserRole,
    pub tenant_id: String,
    pub is_active: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 2))]
    pub name: String,
    pub tenant_id: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginDto {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub project_id: String,
    pub name: String,
    pub description: String,
    pub tenant_id: String,
    pub owner_id: String,
    pub member_ids: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProjectDto {
    #[validate(length(min = 3))]
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalyticsQuery {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub query_id: String,
    pub project_id: String,
    pub user_id: String,
    pub query_text: String,
    pub response_text: Option<String>,
    pub status: QueryStatus,
    pub created_at: DateTime,
    pub completed_at: Option<DateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum QueryStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateQueryDto {
    #[validate(length(min = 3))]
    pub query_text: String,
    pub project_id: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub user_id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub tenant_id: String,
    pub is_active: bool,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            user_id: user.user_id.to_string(),
            email: user.email,
            name: user.name,
            role: user.role.as_str().to_string(),
            tenant_id: user.tenant_id,
            is_active: user.is_active,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub project_id: String,
    pub name: String,
    pub description: String,
    pub tenant_id: String,
    pub owner_id: String,
    pub is_active: bool,
    pub created_at: String,
}

impl From<Project> for ProjectResponse {
    fn from(project: Project) -> Self {
        ProjectResponse {
            project_id: project.project_id.to_string(),
            name: project.name,
            description: project.description,
            tenant_id: project.tenant_id,
            owner_id: project.owner_id.to_string(),
            is_active: project.is_active,
            created_at: project.created_at.to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserResponse,
}

// Chat Models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub timestamp: DateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Conversation {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(with = "uuid_as_string")]
    pub conversation_id: uuid::Uuid,
    #[serde(with = "uuid_as_string")]
    pub project_id: uuid::Uuid,
    #[serde(with = "uuid_as_string")]
    pub user_id: uuid::Uuid,
    pub title: String,
    pub messages: Vec<ChatMessage>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SendMessageDto {
    #[validate(length(min = 1))]
    pub message: String,
    pub project_id: String,
    pub conversation_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub conversation_id: String,
    pub message: ChatMessageResponse,
}

#[derive(Debug, Serialize, Clone)]
pub struct ChatMessageResponse {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct ConversationResponse {
    pub conversation_id: String,
    pub project_id: String,
    pub user_id: String,
    pub title: String,
    pub messages: Vec<ChatMessageResponse>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ChatMessage> for ChatMessageResponse {
    fn from(msg: ChatMessage) -> Self {
        ChatMessageResponse {
            role: msg.role,
            content: msg.content,
            timestamp: msg.timestamp.to_string(),
        }
    }
}

impl From<Conversation> for ConversationResponse {
    fn from(conv: Conversation) -> Self {
        ConversationResponse {
            conversation_id: conv.conversation_id.to_string(),
            project_id: conv.project_id.to_string(),
            user_id: conv.user_id.to_string(),
            title: conv.title,
            messages: conv.messages.into_iter().map(|m| m.into()).collect(),
            created_at: conv.created_at.to_string(),
            updated_at: conv.updated_at.to_string(),
        }
    }
}

// Rendering Module Models

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RenderContent {
    Text { content: String },
    Chart { data: ChartData },
    Equation { latex: String, display: Option<bool> },
    Table { data: TableData },
    Dataset { data: DatasetData },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ChartData {
    pub chart_type: ChartType,
    pub title: Option<String>,
    pub labels: Vec<String>,
    pub datasets: Vec<ChartDataset>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    Bar,
    Line,
    Pie,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ChartDataset {
    pub label: String,
    pub data: Vec<f64>,
    pub background_color: Option<String>,
    pub border_color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DatasetData {
    pub name: String,
    pub description: Option<String>,
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct StructuredResponse {
    pub items: Vec<RenderContent>,
}

impl StructuredResponse {
    /// Validates the structured response content
    pub fn validate_content(&self) -> Result<(), String> {
        if self.items.is_empty() {
            return Err("Response must contain at least one item".to_string());
        }

        for item in &self.items {
            match item {
                RenderContent::Text { content } => {
                    if content.is_empty() {
                        return Err("Text content cannot be empty".to_string());
                    }
                }
                RenderContent::Chart { data } => {
                    if data.labels.is_empty() {
                        return Err("Chart must have at least one label".to_string());
                    }
                    if data.datasets.is_empty() {
                        return Err("Chart must have at least one dataset".to_string());
                    }
                    for dataset in &data.datasets {
                        if dataset.data.len() != data.labels.len() {
                            return Err("Dataset length must match labels length".to_string());
                        }
                    }
                }
                RenderContent::Equation { latex, .. } => {
                    if latex.is_empty() {
                        return Err("Equation latex cannot be empty".to_string());
                    }
                }
                RenderContent::Table { data } => {
                    if data.headers.is_empty() {
                        return Err("Table must have at least one header".to_string());
                    }
                    for row in &data.rows {
                        if row.len() != data.headers.len() {
                            return Err("All table rows must match header length".to_string());
                        }
                    }
                }
                RenderContent::Dataset { data } => {
                    if data.columns.is_empty() {
                        return Err("Dataset must have at least one column".to_string());
                    }
                    for row in &data.rows {
                        if row.len() != data.columns.len() {
                            return Err("All dataset rows must match column length".to_string());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
