export enum UserRole {
  ADMIN = 'admin',
  PROJECT_OWNER = 'project_owner',
  PROJECT_MEMBER = 'project_member',
  VIEWER = 'viewer'
}

export interface User {
  user_id: string;
  email: string;
  name: string;
  role: UserRole;
  tenant_id: string;
  is_active: boolean;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface RegisterRequest {
  email: string;
  password: string;
  name: string;
  tenant_id: string;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  user: User;
}

export interface Project {
  project_id: string;
  name: string;
  description: string;
  tenant_id: string;
  owner_id: string;
  is_active: boolean;
  created_at: string;
}

export interface CreateProjectRequest {
  name: string;
  description: string;
}

export interface AnalyticsQuery {
  query_id: string;
  project_id: string;
  user_id: string;
  query_text: string;
  response_text?: string;
  status: QueryStatus;
  created_at: string;
  completed_at?: string;
}

export enum QueryStatus {
  PENDING = 'Pending',
  PROCESSING = 'Processing',
  COMPLETED = 'Completed',
  FAILED = 'Failed'
}

export interface CreateQueryRequest {
  query_text: string;
  project_id: string;
}

export interface ApiError {
  error: string;
}
