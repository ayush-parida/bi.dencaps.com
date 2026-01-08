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
            project_id: Uuid::new_v4().to_string(),
            name: dto.name,
            description: dto.description,
            tenant_id: tenant_id.to_string(),
            owner_id: owner_id.to_string(),
            member_ids: vec![],
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        self.db
            .projects_collection()
            .insert_one(&project)
            .await
            .map_err(|e| format!("Failed to create project: {}", e))?;

        Ok(ProjectResponse::from(project))
    }

    pub async fn get_project_by_id(&self, project_id: &Uuid) -> Result<Project, String> {
        let uuid_str = project_id.to_string();
        
        self.db
            .projects_collection()
            .find_one(doc! { "project_id": &uuid_str })
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "Project not found".to_string())
    }

    pub async fn get_projects_by_tenant(&self, tenant_id: &str) -> Result<Vec<ProjectResponse>, String> {
        use futures::stream::TryStreamExt;

        let cursor = self.db
            .projects_collection()
            .find(doc! { "tenant_id": tenant_id })
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
        
        let uuid_str = user_id.to_string();

        // Log for debugging
        log::info!("Querying projects for user_id: {}, tenant_id: {}", uuid_str, tenant_id);

        let cursor = self.db
            .projects_collection()
            .find(
                doc! {
                    "tenant_id": tenant_id,
                    "$or": [
                        { "owner_id": uuid_str.clone() },
                        { "member_ids": uuid_str.clone() }
                    ]
                }
            )
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let projects: Vec<Project> = cursor
            .try_collect()
            .await
            .map_err(|e| format!("Failed to fetch projects: {}", e))?;

        Ok(projects.into_iter().map(ProjectResponse::from).collect())
    }

    pub async fn check_user_access(
        &self,
        project_id: &Uuid,
        user_id: &Uuid,
        role: &str,
        tenant_id: &str,
    ) -> Result<bool, String> {
        let project = self.get_project_by_id(project_id).await?;
        
        // Admin users can access any project in their tenant
        if role == "admin" && project.tenant_id == tenant_id {
            return Ok(true);
        }
        
        let user_id_str = user_id.to_string();
        Ok(project.owner_id == user_id_str || 
           project.member_ids.contains(&user_id_str))
    }
}
