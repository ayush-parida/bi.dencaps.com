use mongodb::bson::{doc, DateTime};
use uuid::Uuid;
use crate::db::DatabaseManager;
use crate::models::{Project, CreateProjectDto, ProjectResponse};

pub struct ProjectService {
    db: DatabaseManager,
}

impl ProjectService {
    pub fn new(db: DatabaseManager) -> Self {
        ProjectService { db }
    }

    pub async fn create_project(
        &self,
        dto: CreateProjectDto,
        owner_id: &Uuid,
        tenant_id: &str,
    ) -> Result<ProjectResponse, String> {
        let now = DateTime::now();

        let project = Project {
            id: None,
            project_id: Uuid::new_v4(),
            name: dto.name,
            description: dto.description,
            tenant_id: tenant_id.to_string(),
            owner_id: *owner_id,
            member_ids: vec![],
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        self.db
            .projects_collection()
            .insert_one(&project, None)
            .await
            .map_err(|e| format!("Failed to create project: {}", e))?;

        Ok(ProjectResponse::from(project))
    }

    pub async fn get_project_by_id(&self, project_id: &Uuid) -> Result<Project, String> {
        self.db
            .projects_collection()
            .find_one(doc! { "project_id": project_id }, None)
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "Project not found".to_string())
    }

    pub async fn get_projects_by_tenant(&self, tenant_id: &str) -> Result<Vec<ProjectResponse>, String> {
        use futures::stream::TryStreamExt;

        let cursor = self.db
            .projects_collection()
            .find(doc! { "tenant_id": tenant_id }, None)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let projects: Vec<Project> = cursor
            .try_collect()
            .await
            .map_err(|e| format!("Failed to fetch projects: {}", e))?;

        Ok(projects.into_iter().map(ProjectResponse::from).collect())
    }

    pub async fn get_user_projects(
        &self,
        user_id: &Uuid,
        tenant_id: &str,
    ) -> Result<Vec<ProjectResponse>, String> {
        use futures::stream::TryStreamExt;

        let cursor = self.db
            .projects_collection()
            .find(
                doc! {
                    "tenant_id": tenant_id,
                    "$or": [
                        { "owner_id": user_id },
                        { "member_ids": user_id.to_string() }
                    ]
                },
                None,
            )
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let projects: Vec<Project> = cursor
            .try_collect()
            .await
            .map_err(|e| format!("Failed to fetch projects: {}", e))?;

        Ok(projects.into_iter().map(ProjectResponse::from).collect())
    }

    pub async fn check_user_access(&self, project_id: &Uuid, user_id: &Uuid) -> Result<bool, String> {
        let project = self.get_project_by_id(project_id).await?;
        
        Ok(project.owner_id == *user_id || 
           project.member_ids.contains(&user_id.to_string()))
    }
}
