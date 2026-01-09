import { Routes } from '@angular/router';
import { authGuard } from './core/guards/auth.guard';
import { permissionGuard } from './core/guards/permission.guard';
import { Permission } from './core/models/permission.model';

export const routes: Routes = [
  {
    path: '',
    redirectTo: 'dashboard',
    pathMatch: 'full'
  },
  {
    path: 'auth',
    children: [
      {
        path: 'login',
        loadComponent: () => import('./features/auth/login/login').then(m => m.Login)
      },
      {
        path: 'register',
        loadComponent: () => import('./features/auth/register/register').then(m => m.Register)
      }
    ]
  },
  {
    path: '',
    canActivate: [authGuard],
    loadComponent: () => import('./shared/layout/layout').then(m => m.Layout),
    children: [
      {
        path: 'dashboard',
        loadComponent: () => import('./features/dashboard/dashboard').then(m => m.Dashboard)
      },
      {
        path: 'projects',
        loadComponent: () => import('./features/projects/project-list/project-list').then(m => m.ProjectList)
      },
      {
        path: 'chat',
        loadComponent: () => import('./features/chat/chat-interface/chat-interface').then(m => m.ChatInterfaceComponent)
      },
      {
        path: 'users',
        loadComponent: () => import('./features/users/user-list').then(m => m.UserListComponent),
        canActivate: [permissionGuard],
        data: { permission: Permission.UserRead }
      }
    ]
  },
  {
    path: '**',
    redirectTo: 'dashboard'
  }
];
