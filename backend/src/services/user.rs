use mongodb::bson::{doc, DateTime};
use uuid::Uuid;
use crate::db::DatabaseManager;
use crate::models::{User, CreateUserDto, UserRole, UserResponse};
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
            .find_one(doc! { "email": &dto.email }, None)
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
            role: UserRole::Viewer,
            tenant_id: dto.tenant_id,
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        self.db
            .users_collection()
            .insert_one(&user, None)
            .await
            .map_err(|e| format!("Failed to create user: {}", e))?;

        Ok(UserResponse::from(user))
    }

    pub async fn authenticate(&self, email: &str, password: &str) -> Result<User, String> {
        let user = self.db
            .users_collection()
            .find_one(doc! { "email": email }, None)
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
            .find_one(doc! { "user_id": uuid_str }, None)
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "User not found".to_string())
    }

    pub async fn get_users_by_tenant(&self, tenant_id: &str) -> Result<Vec<UserResponse>, String> {
        use futures::stream::TryStreamExt;

        let cursor = self.db
            .users_collection()
            .find(doc! { "tenant_id": tenant_id }, None)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let users: Vec<User> = cursor
            .try_collect()
            .await
            .map_err(|e| format!("Failed to fetch users: {}", e))?;

        Ok(users.into_iter().map(UserResponse::from).collect())
    }
}
