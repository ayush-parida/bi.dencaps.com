/**
 * Permission enum matching backend Permission enum
 */
export enum Permission {
  // Project permissions
  ProjectCreate = 'project:create',
  ProjectRead = 'project:read',
  ProjectUpdate = 'project:update',
  ProjectDelete = 'project:delete',
  ProjectManageMembers = 'project:manage_members',

  // User permissions
  UserCreate = 'user:create',
  UserRead = 'user:read',
  UserUpdate = 'user:update',
  UserDelete = 'user:delete',
  UserManageRoles = 'user:manage_roles',

  // Chat permissions
  ChatRead = 'chat:read',
  ChatWrite = 'chat:write',
  ChatDelete = 'chat:delete',
  ChatExport = 'chat:export',

  // Report permissions
  ReportCreate = 'report:create',
  ReportRead = 'report:read',
  ReportExport = 'report:export',
  ReportDelete = 'report:delete',

  // Admin permissions
  AdminAccess = 'admin:access',
  SystemSettings = 'system:settings'
}

/**
 * Role model matching backend Role struct
 */
export interface Role {
  role_id: string;
  name: string;
  description: string;
  permissions: string[];
  is_system_role: boolean;
  tenant_id: string;
  created_at: string;
  updated_at: string;
}

/**
 * Project membership model
 */
export interface ProjectMembership {
  membership_id: string;
  user_id: string;
  project_id: string;
  role_id: string;
  tenant_id: string;
  assigned_at: string;
  assigned_by: string;
}

/**
 * Resolved permissions for a user (optionally within a project context)
 */
export interface ResolvedPermissions {
  user_id: string;
  tenant_id: string;
  project_id?: string;
  permissions: string[];
  is_admin: boolean;
}

/**
 * DTO for creating a role
 */
export interface CreateRoleRequest {
  name: string;
  description: string;
  permissions: string[];
}

/**
 * DTO for updating a role
 */
export interface UpdateRoleRequest {
  name?: string;
  description?: string;
  permissions?: string[];
}

/**
 * DTO for assigning a role to a user
 */
export interface AssignRoleRequest {
  user_id: string;
  project_id: string;
  role_id: string;
}

/**
 * Response for user permissions endpoint
 * Matches backend UserPermissionsResponse structure
 */
export interface UserPermissionsResponse {
  user_id: string;
  project_id: string | null;
  permissions: string[];
  is_admin: boolean;
}

/**
 * Project member details for display
 */
export interface ProjectMember {
  user_id: string;
  email: string;
  name: string;
  role_id: string;
  role_name: string;
  assigned_at: string;
}
