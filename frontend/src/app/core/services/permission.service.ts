import { Injectable, inject, signal, computed } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, tap, catchError, throwError, of, BehaviorSubject } from 'rxjs';
import { environment } from '../../../environments/environment';
import {
  Permission,
  Role,
  ResolvedPermissions,
  UserPermissionsResponse,
  CreateRoleRequest,
  UpdateRoleRequest,
  AssignRoleRequest,
  ProjectMembership,
  ProjectMember
} from '../models';

@Injectable({
  providedIn: 'root'
})
export class PermissionService {
  private readonly http = inject(HttpClient);
  private readonly apiUrl = `${environment.apiUrl}/rbac`;

  // Cache for resolved permissions per project (now stores UserPermissionsResponse directly)
  private readonly permissionsCache = new Map<string, UserPermissionsResponse>();
  private readonly cacheTimeout = 5 * 60 * 1000; // 5 minutes
  private readonly cacheTimestamps = new Map<string, number>();

  // Signal-based reactive state for current permissions
  private readonly currentPermissionsSubject = new BehaviorSubject<UserPermissionsResponse | null>(null);
  public readonly currentPermissions$ = this.currentPermissionsSubject.asObservable();

  // Signal for current project context
  private readonly currentProjectId = signal<string | null>(null);

  /**
   * Get all available permissions (for role management UI)
   */
  getAllPermissions(): Observable<string[]> {
    return this.http.get<string[]>(`${this.apiUrl}/permissions`);
  }

  /**
   * Get current user's permissions, optionally for a specific project
   */
  getMyPermissions(projectId?: string): Observable<UserPermissionsResponse> {
    let params: Record<string, string> = {};
    if (projectId) {
      params['project_id'] = projectId;
    }
    return this.http.get<UserPermissionsResponse>(`${this.apiUrl}/permissions/me`, { params })
      .pipe(
        tap(response => {
          const cacheKey = projectId || 'global';
          // Store the full response which includes is_admin and permissions
          this.permissionsCache.set(cacheKey, response);
          this.cacheTimestamps.set(cacheKey, Date.now());
          this.currentPermissionsSubject.next(response);
        }),
        catchError(this.handleError)
      );
  }

  /**
   * Check if current user has a specific permission
   * Uses cached permissions when available
   */
  hasPermission(permission: Permission, projectId?: string): boolean {
    const cacheKey = projectId || 'global';
    const cached = this.permissionsCache.get(cacheKey);
    const timestamp = this.cacheTimestamps.get(cacheKey);

    if (cached && timestamp && (Date.now() - timestamp) < this.cacheTimeout) {
      return cached.is_admin || cached.permissions.includes(permission);
    }

    // If no cache, return false - caller should ensure permissions are loaded
    return false;
  }

  /**
   * Check if user has any of the given permissions
   */
  hasAnyPermission(permissions: Permission[], projectId?: string): boolean {
    const cacheKey = projectId || 'global';
    const cached = this.permissionsCache.get(cacheKey);

    if (cached) {
      if (cached.is_admin) return true;
      return permissions.some(p => cached.permissions.includes(p));
    }

    return false;
  }

  /**
   * Check if user has all of the given permissions
   */
  hasAllPermissions(permissions: Permission[], projectId?: string): boolean {
    const cacheKey = projectId || 'global';
    const cached = this.permissionsCache.get(cacheKey);

    if (cached) {
      if (cached.is_admin) return true;
      return permissions.every(p => cached.permissions.includes(p));
    }

    return false;
  }

  /**
   * Check if current user is an admin
   */
  isAdmin(): boolean {
    const cached = this.permissionsCache.get('global') || this.currentPermissionsSubject.value;
    return cached?.is_admin ?? false;
  }

  /**
   * Set current project context for permission checking
   */
  setProjectContext(projectId: string | null): void {
    this.currentProjectId.set(projectId);
    if (projectId) {
      // Load permissions for this project if not cached
      const cacheKey = projectId;
      const timestamp = this.cacheTimestamps.get(cacheKey);
      if (!timestamp || (Date.now() - timestamp) >= this.cacheTimeout) {
        this.getMyPermissions(projectId).subscribe();
      }
    }
  }

  /**
   * Clear all cached permissions (call on logout)
   */
  clearCache(): void {
    this.permissionsCache.clear();
    this.cacheTimestamps.clear();
    this.currentPermissionsSubject.next(null);
    this.currentProjectId.set(null);
  }

  /**
   * Invalidate cache for a specific project
   */
  invalidateProjectCache(projectId: string): void {
    this.permissionsCache.delete(projectId);
    this.cacheTimestamps.delete(projectId);
  }

  // ==================== Role Management ====================

  /**
   * Get all roles for current tenant
   */
  getRoles(): Observable<Role[]> {
    return this.http.get<Role[]>(`${this.apiUrl}/roles`);
  }

  /**
   * Get a specific role by ID
   */
  getRole(roleId: string): Observable<Role> {
    return this.http.get<Role>(`${this.apiUrl}/roles/${roleId}`);
  }

  /**
   * Create a new role
   */
  createRole(request: CreateRoleRequest): Observable<Role> {
    return this.http.post<Role>(`${this.apiUrl}/roles`, request);
  }

  /**
   * Update an existing role
   */
  updateRole(roleId: string, request: UpdateRoleRequest): Observable<Role> {
    return this.http.put<Role>(`${this.apiUrl}/roles/${roleId}`, request);
  }

  /**
   * Delete a role
   */
  deleteRole(roleId: string): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/roles/${roleId}`);
  }

  // ==================== Membership Management ====================

  /**
   * Get current user's memberships
   */
  getMyMemberships(): Observable<ProjectMembership[]> {
    return this.http.get<ProjectMembership[]>(`${this.apiUrl}/memberships/me`);
  }

  /**
   * Get a specific user's memberships (admin only)
   */
  getUserMemberships(userId: string): Observable<ProjectMembership[]> {
    return this.http.get<ProjectMembership[]>(`${this.apiUrl}/memberships/user/${userId}`);
  }

  /**
   * Assign a role to a user for a project
   */
  assignRole(request: AssignRoleRequest): Observable<ProjectMembership> {
    return this.http.post<ProjectMembership>(`${this.apiUrl}/memberships`, request)
      .pipe(
        tap(() => {
          // Invalidate cache for the affected project
          this.invalidateProjectCache(request.project_id);
        })
      );
  }

  /**
   * Revoke a user's role from a project
   */
  revokeRole(projectId: string, userId: string): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/memberships/${projectId}/${userId}`)
      .pipe(
        tap(() => {
          this.invalidateProjectCache(projectId);
        })
      );
  }

  /**
   * Get all members of a project
   */
  getProjectMembers(projectId: string): Observable<ProjectMember[]> {
    return this.http.get<ProjectMember[]>(`${this.apiUrl}/projects/${projectId}/members`);
  }

  /**
   * Initialize system roles for the tenant (admin only)
   */
  initializeSystemRoles(): Observable<Role[]> {
    return this.http.post<Role[]>(`${this.apiUrl}/initialize`, {});
  }

  // ==================== Utilities ====================

  /**
   * Get a human-readable permission label
   */
  getPermissionLabel(permission: Permission): string {
    const labels: Record<Permission, string> = {
      [Permission.ProjectCreate]: 'Create Projects',
      [Permission.ProjectRead]: 'View Projects',
      [Permission.ProjectUpdate]: 'Edit Projects',
      [Permission.ProjectDelete]: 'Delete Projects',
      [Permission.ProjectManageMembers]: 'Manage Project Members',
      [Permission.UserCreate]: 'Create Users',
      [Permission.UserRead]: 'View Users',
      [Permission.UserUpdate]: 'Edit Users',
      [Permission.UserDelete]: 'Delete Users',
      [Permission.UserManageRoles]: 'Manage User Roles',
      [Permission.ChatRead]: 'View Chat History',
      [Permission.ChatWrite]: 'Send Chat Messages',
      [Permission.ChatDelete]: 'Delete Chat Messages',
      [Permission.ChatExport]: 'Export Chat History',
      [Permission.ReportCreate]: 'Create Reports',
      [Permission.ReportRead]: 'View Reports',
      [Permission.ReportExport]: 'Export Reports',
      [Permission.ReportDelete]: 'Delete Reports',
      [Permission.AdminAccess]: 'Admin Access',
      [Permission.SystemSettings]: 'System Settings'
    };
    return labels[permission] || permission;
  }

  /**
   * Group permissions by category for UI display
   */
  getPermissionsByCategory(): Record<string, Permission[]> {
    return {
      'Project': [
        Permission.ProjectCreate,
        Permission.ProjectRead,
        Permission.ProjectUpdate,
        Permission.ProjectDelete,
        Permission.ProjectManageMembers
      ],
      'User': [
        Permission.UserCreate,
        Permission.UserRead,
        Permission.UserUpdate,
        Permission.UserDelete,
        Permission.UserManageRoles
      ],
      'Chat': [
        Permission.ChatRead,
        Permission.ChatWrite,
        Permission.ChatDelete,
        Permission.ChatExport
      ],
      'Reports': [
        Permission.ReportCreate,
        Permission.ReportRead,
        Permission.ReportExport,
        Permission.ReportDelete
      ],
      'Admin': [
        Permission.AdminAccess,
        Permission.SystemSettings
      ]
    };
  }

  private handleError = (error: any): Observable<never> => {
    console.error('Permission service error:', error);
    return throwError(() => error);
  };
}
