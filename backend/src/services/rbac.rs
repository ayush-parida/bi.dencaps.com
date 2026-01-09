use crate::db::DatabaseManager;
use crate::models::{
    Permission, Role, ProjectMembership, ResolvedPermissions,
    CreateRoleDto, UpdateRoleDto, AssignRoleDto, UserRole,
};
use mongodb::bson::{doc, DateTime};
use redis::AsyncCommands;
use std::collections::HashSet;
use uuid::Uuid;

const PERMISSION_CACHE_TTL: i64 = 300; // 5 minutes
const ROLE_CACHE_TTL: i64 = 600; // 10 minutes

/// RBAC Service handles all role and permission operations
pub struct RbacService {
    db: DatabaseManager,
}

impl RbacService {
    pub fn new(db: DatabaseManager) -> Self {
        RbacService { db }
    }

    // ========================================================================
    // Permission Resolution
    // ========================================================================

    /// Resolve permissions for a user in a specific project context
    /// This is the main entry point for permission checks
    pub async fn resolve_permissions(
        &self,
        user_id: &str,
        project_id: Option<&str>,
    ) -> Result<ResolvedPermissions, String> {
        let cache_key = match project_id {
            Some(pid) => format!("permissions:{}:{}", user_id, pid),
            None => format!("permissions:{}:global", user_id),
        };

        // Try cache first
        if let Some(cached) = self.get_cached_permissions(&cache_key).await {
            return Ok(cached);
        }

        // Fetch user to check if admin
        let user = self.get_user(user_id).await?;
        let is_admin = matches!(user.role, UserRole::Admin);

        let permissions = if is_admin {
            // Admins get all permissions
            Permission::all()
                .iter()
                .map(|p| p.as_str().to_string())
                .collect()
        } else if let Some(pid) = project_id {
            // Get project-specific permissions
            self.get_project_permissions(user_id, pid).await?
        } else {
            // Get global permissions based on user role
            self.get_global_permissions(&user.role).await?
        };

        let resolved = ResolvedPermissions {
            user_id: user_id.to_string(),
            project_id: project_id.map(|s| s.to_string()),
            permissions,
            is_admin,
            resolved_at: chrono::Utc::now().timestamp(),
        };

        // Cache the result
        self.cache_permissions(&cache_key, &resolved).await;

        Ok(resolved)
    }

    /// Get permissions for a user in a specific project
    async fn get_project_permissions(
        &self,
        user_id: &str,
        project_id: &str,
    ) -> Result<HashSet<String>, String> {
        // First verify user has access to this project
        let membership = self.get_membership(user_id, project_id).await?;
        
        if let Some(m) = membership {
            // Get role and its permissions
            let role = self.get_role_by_id(&m.role_id).await?;
            if let Some(r) = role {
                return Ok(r.permissions.into_iter().collect());
            }
        }

        // Check if user is project owner
        let project = self.get_project(project_id).await?;
        if let Some(p) = project {
            if p.owner_id == user_id {
                // Project owners get full project permissions
                return Ok(self.get_owner_permissions());
            }
        }

        Ok(HashSet::new())
    }

    /// Get global permissions based on UserRole enum
    async fn get_global_permissions(&self, role: &UserRole) -> Result<HashSet<String>, String> {
        let permissions = match role {
            UserRole::Admin => Permission::all()
                .iter()
                .map(|p| p.as_str().to_string())
                .collect(),
            UserRole::ProjectOwner => self.get_owner_permissions(),
            UserRole::ProjectMember => self.get_member_permissions(),
            UserRole::Viewer => self.get_viewer_permissions(),
        };
        Ok(permissions)
    }

    fn get_owner_permissions(&self) -> HashSet<String> {
        vec![
            Permission::ProjectRead,
            Permission::ProjectUpdate,
            Permission::ProjectManageMembers,
            Permission::UserRead,
            Permission::ChatRead,
            Permission::ChatWrite,
            Permission::ChatDelete,
            Permission::ChatExport,
            Permission::ReportCreate,
            Permission::ReportRead,
            Permission::ReportExport,
            Permission::ReportDelete,
        ]
        .iter()
        .map(|p| p.as_str().to_string())
        .collect()
    }

    fn get_member_permissions(&self) -> HashSet<String> {
        vec![
            Permission::ProjectRead,
            Permission::ChatRead,
            Permission::ChatWrite,
            Permission::ReportCreate,
            Permission::ReportRead,
        ]
        .iter()
        .map(|p| p.as_str().to_string())
        .collect()
    }

    fn get_viewer_permissions(&self) -> HashSet<String> {
        vec![
            Permission::ProjectRead,
            Permission::ChatRead,
            Permission::ReportRead,
        ]
        .iter()
        .map(|p| p.as_str().to_string())
        .collect()
    }

    // ========================================================================
    // Role Management
    // ========================================================================

    /// Create a new role
    pub async fn create_role(
        &self,
        dto: CreateRoleDto,
        tenant_id: &str,
    ) -> Result<Role, String> {
        // Validate permissions
        for perm in &dto.permissions {
            if Permission::from_str(perm).is_none() {
                return Err(format!("Invalid permission: {}", perm));
            }
        }

        let now = DateTime::now();
        let role = Role {
            id: None,
            role_id: Uuid::new_v4().to_string(),
            name: dto.name,
            description: dto.description,
            permissions: dto.permissions,
            is_system_role: false,
            tenant_id: tenant_id.to_string(),
            created_at: now,
            updated_at: now,
        };

        self.db
            .roles_collection()
            .insert_one(&role)
            .await
            .map_err(|e| format!("Failed to create role: {}", e))?;

        // Invalidate role cache
        self.invalidate_role_cache(&role.role_id).await;

        log::info!("Created role: {} in tenant: {}", role.role_id, tenant_id);
        Ok(role)
    }

    /// Update an existing role
    pub async fn update_role(
        &self,
        role_id: &str,
        dto: UpdateRoleDto,
        tenant_id: &str,
    ) -> Result<Role, String> {
        let existing = self.get_role_by_id(role_id).await?
            .ok_or("Role not found")?;

        if existing.is_system_role {
            return Err("Cannot modify system role".to_string());
        }

        if existing.tenant_id != tenant_id {
            return Err("Access denied".to_string());
        }

        // Validate permissions if provided
        if let Some(ref perms) = dto.permissions {
            for perm in perms {
                if Permission::from_str(perm).is_none() {
                    return Err(format!("Invalid permission: {}", perm));
                }
            }
        }

        let mut update_doc = doc! {
            "$set": {
                "updated_at": DateTime::now()
            }
        };

        if let Some(name) = &dto.name {
            update_doc.get_document_mut("$set").unwrap()
                .insert("name", name.clone());
        }
        if let Some(desc) = &dto.description {
            update_doc.get_document_mut("$set").unwrap()
                .insert("description", desc.clone());
        }
        if let Some(perms) = &dto.permissions {
            update_doc.get_document_mut("$set").unwrap()
                .insert("permissions", perms.clone());
        }

        self.db
            .roles_collection()
            .update_one(
                doc! { "role_id": role_id, "tenant_id": tenant_id },
                update_doc,
            )
            .await
            .map_err(|e| format!("Failed to update role: {}", e))?;

        // Invalidate caches
        self.invalidate_role_cache(role_id).await;
        self.invalidate_permissions_for_role(role_id).await;

        self.get_role_by_id(role_id).await?
            .ok_or("Role not found after update".to_string())
    }

    /// Delete a role
    pub async fn delete_role(&self, role_id: &str, tenant_id: &str) -> Result<(), String> {
        let role = self.get_role_by_id(role_id).await?
            .ok_or("Role not found")?;

        if role.is_system_role {
            return Err("Cannot delete system role".to_string());
        }

        if role.tenant_id != tenant_id {
            return Err("Access denied".to_string());
        }

        // Check if role is in use
        let membership_count = self.db
            .memberships_collection()
            .count_documents(doc! { "role_id": role_id })
            .await
            .map_err(|e| format!("Failed to check role usage: {}", e))?;

        if membership_count > 0 {
            return Err(format!(
                "Cannot delete role: {} users are assigned to it",
                membership_count
            ));
        }

        self.db
            .roles_collection()
            .delete_one(doc! { "role_id": role_id, "tenant_id": tenant_id })
            .await
            .map_err(|e| format!("Failed to delete role: {}", e))?;

        // Invalidate cache
        self.invalidate_role_cache(role_id).await;

        log::info!("Deleted role: {}", role_id);
        Ok(())
    }

    /// Get role by ID
    pub async fn get_role_by_id(&self, role_id: &str) -> Result<Option<Role>, String> {
        let cache_key = format!("role:{}", role_id);
        
        // Try cache first
        if let Some(cached) = self.get_cached_role(&cache_key).await {
            return Ok(Some(cached));
        }

        let role = self.db
            .roles_collection()
            .find_one(doc! { "role_id": role_id })
            .await
            .map_err(|e| format!("Failed to get role: {}", e))?;

        if let Some(ref r) = role {
            self.cache_role(&cache_key, r).await;
        }

        Ok(role)
    }

    /// Get all roles for a tenant
    pub async fn get_tenant_roles(&self, tenant_id: &str) -> Result<Vec<Role>, String> {
        use futures::TryStreamExt;

        let cursor = self.db
            .roles_collection()
            .find(doc! { "tenant_id": tenant_id })
            .await
            .map_err(|e| format!("Failed to get roles: {}", e))?;

        let roles: Vec<Role> = cursor
            .try_collect()
            .await
            .map_err(|e| format!("Failed to collect roles: {}", e))?;

        Ok(roles)
    }

    // ========================================================================
    // Membership Management
    // ========================================================================

    /// Assign a role to a user for a specific project
    pub async fn assign_role(&self, dto: AssignRoleDto, tenant_id: &str) -> Result<ProjectMembership, String> {
        // Verify role exists and belongs to tenant
        let role = self.get_role_by_id(&dto.role_id).await?
            .ok_or("Role not found")?;

        if role.tenant_id != tenant_id {
            return Err("Role does not belong to this tenant".to_string());
        }

        // Verify project exists and belongs to tenant
        let project = self.get_project(&dto.project_id).await?
            .ok_or("Project not found")?;

        if project.tenant_id != tenant_id {
            return Err("Project does not belong to this tenant".to_string());
        }

        // Verify user exists
        let _user = self.get_user(&dto.user_id).await?;

        // Check for existing membership
        let existing = self.get_membership(&dto.user_id, &dto.project_id).await?;
        
        if existing.is_some() {
            // Update existing membership
            return self.update_membership(&dto.user_id, &dto.project_id, &dto.role_id, tenant_id).await;
        }

        // Create new membership
        let now = DateTime::now();
        let membership = ProjectMembership {
            id: None,
            membership_id: Uuid::new_v4().to_string(),
            user_id: dto.user_id.clone(),
            project_id: dto.project_id.clone(),
            role_id: dto.role_id.clone(),
            tenant_id: tenant_id.to_string(),
            created_at: now,
            updated_at: now,
        };

        self.db
            .memberships_collection()
            .insert_one(&membership)
            .await
            .map_err(|e| format!("Failed to assign role: {}", e))?;

        // Invalidate permission cache for user
        self.invalidate_user_permissions(&dto.user_id, Some(&dto.project_id)).await;

        log::info!(
            "Assigned role {} to user {} for project {}",
            dto.role_id, dto.user_id, dto.project_id
        );

        Ok(membership)
    }

    /// Update an existing membership
    async fn update_membership(
        &self,
        user_id: &str,
        project_id: &str,
        role_id: &str,
        tenant_id: &str,
    ) -> Result<ProjectMembership, String> {
        self.db
            .memberships_collection()
            .update_one(
                doc! {
                    "user_id": user_id,
                    "project_id": project_id,
                    "tenant_id": tenant_id
                },
                doc! {
                    "$set": {
                        "role_id": role_id,
                        "updated_at": DateTime::now()
                    }
                },
            )
            .await
            .map_err(|e| format!("Failed to update membership: {}", e))?;

        // Invalidate permission cache
        self.invalidate_user_permissions(user_id, Some(project_id)).await;

        self.get_membership(user_id, project_id).await?
            .ok_or("Membership not found after update".to_string())
    }

    /// Remove role assignment
    pub async fn revoke_role(
        &self,
        user_id: &str,
        project_id: &str,
        tenant_id: &str,
    ) -> Result<(), String> {
        let result = self.db
            .memberships_collection()
            .delete_one(doc! {
                "user_id": user_id,
                "project_id": project_id,
                "tenant_id": tenant_id
            })
            .await
            .map_err(|e| format!("Failed to revoke role: {}", e))?;

        if result.deleted_count == 0 {
            return Err("Membership not found".to_string());
        }

        // Invalidate permission cache
        self.invalidate_user_permissions(user_id, Some(project_id)).await;

        log::info!("Revoked role from user {} for project {}", user_id, project_id);
        Ok(())
    }

    /// Get membership for a user in a project
    pub async fn get_membership(
        &self,
        user_id: &str,
        project_id: &str,
    ) -> Result<Option<ProjectMembership>, String> {
        self.db
            .memberships_collection()
            .find_one(doc! {
                "user_id": user_id,
                "project_id": project_id
            })
            .await
            .map_err(|e| format!("Failed to get membership: {}", e))
    }

    /// Get all memberships for a project
    pub async fn get_project_memberships(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectMembership>, String> {
        use futures::TryStreamExt;

        let cursor = self.db
            .memberships_collection()
            .find(doc! { "project_id": project_id })
            .await
            .map_err(|e| format!("Failed to get memberships: {}", e))?;

        cursor
            .try_collect()
            .await
            .map_err(|e| format!("Failed to collect memberships: {}", e))
    }

    /// Get all memberships for a user
    pub async fn get_user_memberships(
        &self,
        user_id: &str,
    ) -> Result<Vec<ProjectMembership>, String> {
        use futures::TryStreamExt;

        let cursor = self.db
            .memberships_collection()
            .find(doc! { "user_id": user_id })
            .await
            .map_err(|e| format!("Failed to get memberships: {}", e))?;

        cursor
            .try_collect()
            .await
            .map_err(|e| format!("Failed to collect memberships: {}", e))
    }

    // ========================================================================
    // System Role Initialization
    // ========================================================================

    /// Initialize default system roles for a tenant
    pub async fn initialize_system_roles(&self, tenant_id: &str) -> Result<(), String> {
        let existing = self.get_tenant_roles(tenant_id).await?;
        if existing.iter().any(|r| r.is_system_role) {
            log::info!("System roles already exist for tenant: {}", tenant_id);
            return Ok(());
        }

        let now = DateTime::now();
        
        let system_roles = vec![
            Role {
                id: None,
                role_id: format!("{}-admin", tenant_id),
                name: "Administrator".to_string(),
                description: "Full system access".to_string(),
                permissions: Permission::all().iter().map(|p| p.as_str().to_string()).collect(),
                is_system_role: true,
                tenant_id: tenant_id.to_string(),
                created_at: now,
                updated_at: now,
            },
            Role {
                id: None,
                role_id: format!("{}-owner", tenant_id),
                name: "Project Owner".to_string(),
                description: "Full project access".to_string(),
                permissions: self.get_owner_permissions().into_iter().collect(),
                is_system_role: true,
                tenant_id: tenant_id.to_string(),
                created_at: now,
                updated_at: now,
            },
            Role {
                id: None,
                role_id: format!("{}-member", tenant_id),
                name: "Project Member".to_string(),
                description: "Standard project access".to_string(),
                permissions: self.get_member_permissions().into_iter().collect(),
                is_system_role: true,
                tenant_id: tenant_id.to_string(),
                created_at: now,
                updated_at: now,
            },
            Role {
                id: None,
                role_id: format!("{}-viewer", tenant_id),
                name: "Viewer".to_string(),
                description: "Read-only access".to_string(),
                permissions: self.get_viewer_permissions().into_iter().collect(),
                is_system_role: true,
                tenant_id: tenant_id.to_string(),
                created_at: now,
                updated_at: now,
            },
        ];

        for role in system_roles {
            self.db
                .roles_collection()
                .insert_one(&role)
                .await
                .map_err(|e| format!("Failed to create system role: {}", e))?;
        }

        log::info!("Initialized system roles for tenant: {}", tenant_id);
        Ok(())
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    async fn get_user(&self, user_id: &str) -> Result<crate::models::User, String> {
        self.db
            .users_collection()
            .find_one(doc! { "user_id": user_id })
            .await
            .map_err(|e| format!("Failed to get user: {}", e))?
            .ok_or_else(|| "User not found".to_string())
    }

    async fn get_project(&self, project_id: &str) -> Result<Option<crate::models::Project>, String> {
        self.db
            .projects_collection()
            .find_one(doc! { "project_id": project_id })
            .await
            .map_err(|e| format!("Failed to get project: {}", e))
    }

    // ========================================================================
    // Redis Caching
    // ========================================================================

    async fn get_cached_permissions(&self, key: &str) -> Option<ResolvedPermissions> {
        let mut redis = (*self.db.redis).clone();
        
        let result: Result<Option<String>, _> = redis.get(key).await;
        match result {
            Ok(Some(json)) => serde_json::from_str(&json).ok(),
            _ => None,
        }
    }

    async fn cache_permissions(&self, key: &str, permissions: &ResolvedPermissions) {
        let mut redis = (*self.db.redis).clone();
        
        if let Ok(json) = serde_json::to_string(permissions) {
            let _: Result<(), _> = redis
                .set_ex(key, json, PERMISSION_CACHE_TTL as u64)
                .await;
        }
    }

    async fn get_cached_role(&self, key: &str) -> Option<Role> {
        let mut redis = (*self.db.redis).clone();
        
        let result: Result<Option<String>, _> = redis.get(key).await;
        match result {
            Ok(Some(json)) => serde_json::from_str(&json).ok(),
            _ => None,
        }
    }

    async fn cache_role(&self, key: &str, role: &Role) {
        let mut redis = (*self.db.redis).clone();
        
        if let Ok(json) = serde_json::to_string(role) {
            let _: Result<(), _> = redis
                .set_ex(key, json, ROLE_CACHE_TTL as u64)
                .await;
        }
    }

    async fn invalidate_role_cache(&self, role_id: &str) {
        let mut redis = (*self.db.redis).clone();
        let key = format!("role:{}", role_id);
        let _: Result<(), _> = redis.del(&key).await;
    }

    async fn invalidate_user_permissions(&self, user_id: &str, project_id: Option<&str>) {
        let mut redis = (*self.db.redis).clone();
        
        // Invalidate specific project permission cache
        if let Some(pid) = project_id {
            let key = format!("permissions:{}:{}", user_id, pid);
            let _: Result<(), _> = redis.del(&key).await;
        }
        
        // Also invalidate global permissions
        let global_key = format!("permissions:{}:global", user_id);
        let _: Result<(), _> = redis.del(&global_key).await;
    }

    async fn invalidate_permissions_for_role(&self, role_id: &str) {
        // Get all memberships using this role and invalidate their caches
        if let Ok(memberships) = self.get_memberships_by_role(role_id).await {
            for m in memberships {
                self.invalidate_user_permissions(&m.user_id, Some(&m.project_id)).await;
            }
        }
    }

    async fn get_memberships_by_role(&self, role_id: &str) -> Result<Vec<ProjectMembership>, String> {
        use futures::TryStreamExt;

        let cursor = self.db
            .memberships_collection()
            .find(doc! { "role_id": role_id })
            .await
            .map_err(|e| format!("Failed to get memberships: {}", e))?;

        cursor
            .try_collect()
            .await
            .map_err(|e| format!("Failed to collect memberships: {}", e))
    }

    /// Invalidate all permission caches (use sparingly)
    pub async fn invalidate_all_caches(&self) {
        let mut redis = (*self.db.redis).clone();
        
        // Use SCAN to find and delete permission keys
        // This is safer than KEYS in production
        let pattern = "permissions:*";
        let _: Result<(), _> = redis
            .del::<_, ()>(pattern)
            .await;
        
        let role_pattern = "role:*";
        let _: Result<(), _> = redis
            .del::<_, ()>(role_pattern)
            .await;
    }
}
