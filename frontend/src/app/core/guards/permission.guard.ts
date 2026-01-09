import { inject } from '@angular/core';
import { CanActivateFn, Router, ActivatedRouteSnapshot } from '@angular/router';
import { map, catchError, of, switchMap, take } from 'rxjs';
import { TokenService } from '../services/token.service';
import { PermissionService } from '../services/permission.service';
import { Permission } from '../models';

/**
 * Route data interface for permission configuration
 */
export interface PermissionRouteData {
  permissions?: Permission[];
  requireAll?: boolean; // If true, requires ALL permissions; if false, requires ANY
  projectIdParam?: string; // Route param name containing project ID
}

/**
 * Permission-based route guard
 * 
 * Usage in route config:
 * {
 *   path: 'admin',
 *   component: AdminComponent,
 *   canActivate: [permissionGuard],
 *   data: {
 *     permissions: [Permission.AdminAccess],
 *     requireAll: true
 *   }
 * }
 * 
 * For project-scoped permissions:
 * {
 *   path: 'projects/:projectId/settings',
 *   component: ProjectSettingsComponent,
 *   canActivate: [permissionGuard],
 *   data: {
 *     permissions: [Permission.ProjectUpdate],
 *     projectIdParam: 'projectId'
 *   }
 * }
 */
export const permissionGuard: CanActivateFn = (route: ActivatedRouteSnapshot, state) => {
  const tokenService = inject(TokenService);
  const permissionService = inject(PermissionService);
  const router = inject(Router);

  // First check if user is authenticated
  if (!tokenService.isAuthenticated()) {
    router.navigate(['/auth/login'], { queryParams: { returnUrl: state.url } });
    return false;
  }

  // Get permission requirements from route data
  const routeData = route.data as PermissionRouteData;
  const requiredPermissions = routeData.permissions;

  // If no permissions specified, just require authentication
  if (!requiredPermissions || requiredPermissions.length === 0) {
    return true;
  }

  const requireAll = routeData.requireAll ?? false;
  const projectIdParam = routeData.projectIdParam;
  const projectId = projectIdParam ? route.params[projectIdParam] : undefined;

  // Load permissions and check access
  return permissionService.getMyPermissions(projectId).pipe(
    take(1),
    map(response => {
      const hasAccess = requireAll
        ? permissionService.hasAllPermissions(requiredPermissions, projectId)
        : permissionService.hasAnyPermission(requiredPermissions, projectId);

      if (!hasAccess) {
        console.warn('Access denied: insufficient permissions', {
          required: requiredPermissions,
          requireAll,
          projectId
        });
        router.navigate(['/access-denied']);
        return false;
      }

      return true;
    }),
    catchError(error => {
      console.error('Permission check failed:', error);
      router.navigate(['/auth/login']);
      return of(false);
    })
  );
};

/**
 * Admin-only route guard
 * Shorthand for routes that require admin access
 */
export const adminGuard: CanActivateFn = (route, state) => {
  const tokenService = inject(TokenService);
  const permissionService = inject(PermissionService);
  const router = inject(Router);

  if (!tokenService.isAuthenticated()) {
    router.navigate(['/auth/login'], { queryParams: { returnUrl: state.url } });
    return false;
  }

  return permissionService.getMyPermissions().pipe(
    take(1),
    map(() => {
      if (!permissionService.isAdmin()) {
        router.navigate(['/access-denied']);
        return false;
      }
      return true;
    }),
    catchError(() => {
      router.navigate(['/auth/login']);
      return of(false);
    })
  );
};

/**
 * Project access guard
 * Ensures user has at least read access to the project
 */
export const projectAccessGuard: CanActivateFn = (route, state) => {
  const tokenService = inject(TokenService);
  const permissionService = inject(PermissionService);
  const router = inject(Router);

  if (!tokenService.isAuthenticated()) {
    router.navigate(['/auth/login'], { queryParams: { returnUrl: state.url } });
    return false;
  }

  // Get project ID from route - check common param names
  const projectId = route.params['projectId'] || route.params['project_id'] || route.params['id'];

  if (!projectId) {
    console.error('Project access guard: No project ID found in route params');
    router.navigate(['/projects']);
    return false;
  }

  return permissionService.getMyPermissions(projectId).pipe(
    take(1),
    map(() => {
      const hasAccess = permissionService.hasPermission(Permission.ProjectRead, projectId);
      
      if (!hasAccess) {
        console.warn('Access denied to project:', projectId);
        router.navigate(['/projects']);
        return false;
      }

      // Set project context for downstream permission checks
      permissionService.setProjectContext(projectId);
      return true;
    }),
    catchError(error => {
      console.error('Project access check failed:', error);
      router.navigate(['/projects']);
      return of(false);
    })
  );
};
