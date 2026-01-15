use mongodb::bson::{doc, DateTime};
use uuid::Uuid;
use crate::db::DatabaseManager;
use crate::models::{User, CreateUserDto, AdminCreateUserDto, UpdateUserDto, UserResponse};
use crate::utils::{hash_password, verify_password};

pub struct UserService {
    db: DatabaseManager,
}

impl UserService {
    pub fn new(db: DatabaseManager) -> Self {
        UserService { db }
    }

    pub async fn create_user(&self, dto: CreateUserDto) -> Result<UserResponse, String> {
        // Check if user already exists
        let existing = self.db
            .users_collection()
            .find_one(doc! { "email": &dto.email })
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        if existing.is_some() {
            return Err("User with this email already exists".to_string());
        }

        let password_hash = hash_password(&dto.password)?;
        let now = DateTime::now();

        let user = User {
            id: None,
            user_id: Uuid::new_v4(),
            email: dto.email,
            password_hash,
            name: dto.name,
            role: "viewer".to_string(),
            tenant_id: dto.tenant_id,
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        self.db
            .users_collection()
            .insert_one(&user)
            .await
            .map_err(|e| format!("Failed to create user: {}", e))?;

        Ok(UserResponse::from(user))
    }

    /// Admin creates a user with specified role
    pub async fn admin_create_user(
        &self,
        dto: AdminCreateUserDto,
        tenant_id: &str,
    ) -> Result<UserResponse, String> {
        // Check if user already exists
        let existing = self.db
            .users_collection()
            .find_one(doc! { "email": &dto.email })
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        if existing.is_some() {
            return Err("User with this email already exists".to_string());
        }

        let password_hash = hash_password(&dto.password)?;
        let now = DateTime::now();

        // Use provided role or default to viewer
        let role = dto.role.unwrap_or_else(|| "viewer".to_string());

        let user = User {
            id: None,
            user_id: Uuid::new_v4(),
            email: dto.email,
            password_hash,
            name: dto.name,
            role,
            tenant_id: tenant_id.to_string(),
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        self.db
            .users_collection()
            .insert_one(&user)
            .await
            .map_err(|e| format!("Failed to create user: {}", e))?;

        Ok(UserResponse::from(user))
    }

    pub async fn authenticate(&self, email: &str, password: &str) -> Result<User, String> {
        let user = self.db
            .users_collection()
            .find_one(doc! { "email": email })
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "Invalid email or password".to_string())?;

        if !user.is_active {
            return Err("User account is not active".to_string());
        }

        let valid = verify_password(password, &user.password_hash)?;
        if !valid {
            return Err("Invalid email or password".to_string());
        }

        Ok(user)
    }

    pub async fn get_user_by_id(&self, user_id: &Uuid) -> Result<User, String> {
        let uuid_str = user_id.to_string();
        
        self.db
            .users_collection()
            .find_one(doc! { "user_id": uuid_str })
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "User not found".to_string())
    }

    pub async fn get_user_by_id_str(&self, user_id: &str) -> Result<User, String> {
        self.db
            .users_collection()
            .find_one(doc! { "user_id": user_id })
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "User not found".to_string())
    }

    pub async fn get_users_by_tenant(&self, tenant_id: &str) -> Result<Vec<UserResponse>, String> {
        use futures::stream::TryStreamExt;

        let cursor = self.db
            .users_collection()
            .find(doc! { "tenant_id": tenant_id })
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let users: Vec<User> = cursor
            .try_collect()
            .await
            .map_err(|e| format!("Failed to fetch users: {}", e))?;

        Ok(users.into_iter().map(UserResponse::from).collect())
    }

    /// Update a user's profile (name, role, is_active)
    pub async fn update_user(
        &self,
        user_id: &str,
        dto: UpdateUserDto,
        tenant_id: &str,
    ) -> Result<UserResponse, String> {
        let now = DateTime::now();

        // Build update document
        let mut update_doc = doc! { "updated_at": now };
        
        if let Some(name) = dto.name {
            update_doc.insert("name", name);
        }
        if let Some(role_str) = dto.role {
            // Accept any role string - supports both system and custom roles
            update_doc.insert("role", role_str);
        }
        if let Some(is_active) = dto.is_active {
            update_doc.insert("is_active", is_active);
        }

        // Update with tenant check for security
        let result = self.db
            .users_collection()
            .update_one(
                doc! { "user_id": user_id, "tenant_id": tenant_id },
                doc! { "$set": update_doc },
            )
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        if result.matched_count == 0 {
            return Err("User not found".to_string());
        }

        // Fetch and return updated user
        let user = self.get_user_by_id_str(user_id).await?;
        Ok(UserResponse::from(user))
    }

    /// Change user's own password
    pub async fn change_password(
        &self,
        user_id: &str,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), String> {
        let user = self.get_user_by_id_str(user_id).await?;

        // Verify current password
        let valid = verify_password(current_password, &user.password_hash)?;
        if !valid {
            return Err("Current password is incorrect".to_string());
        }

        // Hash new password
        let new_hash = hash_password(new_password)?;
        let now = DateTime::now();

        self.db
            .users_collection()
            .update_one(
                doc! { "user_id": user_id },
                doc! { "$set": { "password_hash": new_hash, "updated_at": now } },
            )
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(())
    }

    /// Admin resets user's password
    pub async fn reset_password(
        &self,
        user_id: &str,
        new_password: &str,
        tenant_id: &str,
    ) -> Result<(), String> {
        let new_hash = hash_password(new_password)?;
        let now = DateTime::now();

        let result = self.db
            .users_collection()
            .update_one(
                doc! { "user_id": user_id, "tenant_id": tenant_id },
                doc! { "$set": { "password_hash": new_hash, "updated_at": now } },
            )
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        if result.matched_count == 0 {
            return Err("User not found".to_string());
        }

        Ok(())
    }

    /// Delete a user (soft delete by setting is_active = false, or hard delete)
    pub async fn delete_user(
        &self,
        user_id: &str,
        tenant_id: &str,
        hard_delete: bool,
    ) -> Result<(), String> {
        if hard_delete {
            let result = self.db
                .users_collection()
                .delete_one(doc! { "user_id": user_id, "tenant_id": tenant_id })
                .await
                .map_err(|e| format!("Database error: {}", e))?;

            if result.deleted_count == 0 {
                return Err("User not found".to_string());
            }
        } else {
            // Soft delete
            let now = DateTime::now();
            let result = self.db
                .users_collection()
                .update_one(
                    doc! { "user_id": user_id, "tenant_id": tenant_id },
                    doc! { "$set": { "is_active": false, "updated_at": now } },
                )
                .await
                .map_err(|e| format!("Database error: {}", e))?;

            if result.matched_count == 0 {
                return Err("User not found".to_string());
            }
        }

        Ok(())
    }

    /// Search users by email or name
    pub async fn search_users(
        &self,
        tenant_id: &str,
        query: &str,
    ) -> Result<Vec<UserResponse>, String> {
        use futures::stream::TryStreamExt;

        let cursor = self.db
            .users_collection()
            .find(doc! {
                "tenant_id": tenant_id,
                "$or": [
                    { "email": { "$regex": query, "$options": "i" } },
                    { "name": { "$regex": query, "$options": "i" } }
                ]
            })
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let users: Vec<User> = cursor
            .try_collect()
            .await
            .map_err(|e| format!("Failed to fetch users: {}", e))?;

        Ok(users.into_iter().map(UserResponse::from).collect())
    }
}
