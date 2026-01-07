use serde::{Deserialize, Serialize};
use mongodb::bson::{oid::ObjectId, DateTime};
use validator::Validate;

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
    pub project_id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub tenant_id: String,
    pub owner_id: uuid::Uuid,
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
    pub query_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
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
