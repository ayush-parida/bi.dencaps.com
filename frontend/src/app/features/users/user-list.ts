import { Component, inject, OnInit, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule, ReactiveFormsModule, FormBuilder, Validators } from '@angular/forms';
import { UserManagementService } from '../../core/services/user-management.service';
import { PermissionService } from '../../core/services/permission.service';
import { ProjectService } from '../../core/services/project.service';
import { Permission, Role, Project, AssignRoleRequest, ProjectMembership } from '../../core/models';
import { HasPermissionDirective } from '../../shared/directives/permission.directive';

interface User {
  user_id: string;
  email: string;
  name: string;
  role: string;
  is_active: boolean;
}

interface ProjectMemberInfo {
  project_id: string;
  project_name: string;
  role_id: string;
  role_name: string;
}

@Component({
  selector: 'app-user-list',
  standalone: true,
  imports: [CommonModule, FormsModule, ReactiveFormsModule, HasPermissionDirective],
  templateUrl: './user-list.html',
  styleUrl: './user-list.scss',
})
export class UserListComponent implements OnInit {
  private readonly userService = inject(UserManagementService);
  private readonly permissionService = inject(PermissionService);
  private readonly projectService = inject(ProjectService);
  private readonly fb = inject(FormBuilder);

  readonly Permission = Permission;

  users = signal<User[]>([]);
  roles = signal<Role[]>([]);
  projects = signal<Project[]>([]);
  loading = signal(false);
  error = signal<string | null>(null);
  successMessage = signal<string | null>(null);
  searchQuery = signal('');

  // Modal states
  showCreateModal = signal(false);
  showEditModal = signal(false);
  showResetPasswordModal = signal(false);
  selectedUser = signal<User | null>(null);
  
  // Edit modal tab: 'details' | 'projects'
  editModalTab = signal<'details' | 'projects'>('details');

  // Computed
  filteredUsers = computed(() => {
    const query = this.searchQuery().toLowerCase();
    if (!query) return this.users();
    return this.users().filter(u => 
      u.name.toLowerCase().includes(query) || 
      u.email.toLowerCase().includes(query)
    );
  });

  activeUsers = computed(() => this.users().filter(u => u.is_active).length);
  inactiveUsers = computed(() => this.users().filter(u => !u.is_active).length);

  // Base system role mappings (user.role field values)
  systemRoleMap: Record<string, string> = {
    'admin': 'Administrator',
    'project_owner': 'Project Owner', 
    'project_member': 'Project Member',
    'viewer': 'Viewer'
  };

  // Computed: All available roles for user assignment (system + custom)
  allRoleOptions = computed(() => {
    const systemRoles = [
      { value: 'admin', label: 'Administrator', description: 'Full system access', isSystem: true },
      { value: 'project_owner', label: 'Project Owner', description: 'Full project access', isSystem: true },
      { value: 'project_member', label: 'Project Member', description: 'Standard project access', isSystem: true },
      { value: 'viewer', label: 'Viewer', description: 'Read-only access', isSystem: true }
    ];
    
    // Add custom roles from database
    const customRoles = this.roles()
      .filter(r => !r.is_system_role)
      .map(r => ({
        value: r.name.toLowerCase().replace(/\s+/g, '_'),
        label: r.name,
        description: r.description || 'Custom role',
        isSystem: false
      }));
    
    return [...systemRoles, ...customRoles];
  });

  // Computed: All available roles for project assignment (includes custom roles)
  projectRoleOptions = computed(() => {
    return this.roles().map(r => ({
      value: r.role_id,
      label: r.name,
      description: r.description,
      permissions: r.permissions,
      isSystem: r.is_system_role
    }));
  });

  // Forms
  createForm = this.fb.group({
    email: ['', [Validators.required, Validators.email]],
    name: ['', [Validators.required, Validators.minLength(2)]],
    password: ['', [Validators.required, Validators.minLength(8)]],
    role: ['viewer', Validators.required]
  });

  editForm = this.fb.group({
    name: ['', [Validators.required, Validators.minLength(2)]],
    role: ['', Validators.required],
    is_active: [true]
  });

  resetPasswordForm = this.fb.group({
    new_password: ['', [Validators.required, Validators.minLength(8)]],
    confirm_password: ['', [Validators.required]]
  });

  // Project assignment - selected project IDs
  selectedProjectIds = signal<string[]>([]);
  
  // User's existing project memberships
  userMemberships = signal<ProjectMemberInfo[]>([]);
  loadingMemberships = signal(false);

  ngOnInit(): void {
    this.loadUsers();
    this.loadRoles();
    this.loadProjects();
  }

  loadUsers(): void {
    this.loading.set(true);
    this.error.set(null);
    this.userService.getUsers().subscribe({
      next: (users: User[]) => {
        this.users.set(users);
        this.loading.set(false);
      },
      error: (err: any) => {
        this.error.set(err.error?.message || 'Failed to load users');
        this.loading.set(false);
      }
    });
  }

  loadRoles(): void {
    this.permissionService.getRoles().subscribe({
      next: (roles: Role[]) => {
        this.roles.set(roles);
      },
      error: (err: any) => {
        console.error('Failed to load roles:', err);
      }
    });
  }

  loadProjects(): void {
    this.projectService.getProjects().subscribe({
      next: (projects: Project[]) => {
        this.projects.set(projects);
      },
      error: (err: any) => {
        console.error('Failed to load projects:', err);
      }
    });
  }

  // Get role display name - supports both system and custom roles
  getRoleDisplayName(roleValue: string): string {
    // First check system role map
    if (this.systemRoleMap[roleValue]) {
      return this.systemRoleMap[roleValue];
    }
    // Then check all role options (includes custom roles)
    const option = this.allRoleOptions().find(r => r.value === roleValue);
    return option ? option.label : roleValue;
  }

  // Get role info from database roles
  getRoleInfo(roleValue: string): Role | undefined {
    // Map user.role enum to system role
    const roleMap: Record<string, string> = {
      'admin': 'sys-admin-role',
      'project_owner': 'sys-project-owner-role',
      'project_member': 'sys-project-member-role',
      'viewer': 'sys-viewer-role'
    };
    const roleId = roleMap[roleValue];
    return this.roles().find(r => r.role_id === roleId);
  }

  // Get role by ID
  getRoleById(roleId: string): Role | undefined {
    return this.roles().find(r => r.role_id === roleId);
  }

  // Show success message
  showSuccess(message: string): void {
    this.successMessage.set(message);
    setTimeout(() => this.successMessage.set(null), 3000);
  }

  dismissError(): void {
    this.error.set(null);
  }

  dismissSuccess(): void {
    this.successMessage.set(null);
  }

  // Create
  openCreateModal(): void {
    this.createForm.reset({ role: 'viewer' });
    this.showCreateModal.set(true);
  }

  closeCreateModal(): void {
    this.showCreateModal.set(false);
    this.createForm.reset();
  }

  submitCreate(): void {
    if (this.createForm.invalid) return;
    const data = this.createForm.value;
    this.userService.createUser({
      email: data.email!,
      name: data.name!,
      password: data.password!,
      role: data.role!
    }).subscribe({
      next: () => {
        this.closeCreateModal();
        this.showSuccess('User created successfully');
        this.loadUsers();
      },
      error: (err: any) => {
        this.error.set(err.error?.message || 'Failed to create user');
      }
    });
  }

  // Edit
  openEditModal(user: User): void {
    this.selectedUser.set(user);
    this.editForm.patchValue({
      name: user.name,
      role: user.role,
      is_active: user.is_active
    });
    this.selectedProjectIds.set([]);
    this.userMemberships.set([]);
    this.editModalTab.set('details');
    this.showEditModal.set(true);
    
    // Load user's existing project memberships
    this.loadUserMemberships(user.user_id);
  }

  loadUserMemberships(userId: string): void {
    this.loadingMemberships.set(true);
    this.permissionService.getUserMemberships(userId).subscribe({
      next: (memberships) => {
        // Map memberships to project info
        const projectInfos: ProjectMemberInfo[] = memberships.map(m => {
          const project = this.projects().find(p => p.project_id === m.project_id);
          return {
            project_id: m.project_id,
            project_name: project?.name || 'Unknown Project',
            role_id: m.role_id,
            role_name: m.role_name || 'Unknown Role'
          };
        });
        this.userMemberships.set(projectInfos);
        
        // Pre-select currently assigned projects
        this.selectedProjectIds.set(memberships.map(m => m.project_id));
        this.loadingMemberships.set(false);
      },
      error: (err) => {
        console.error('Failed to load user memberships:', err);
        this.loadingMemberships.set(false);
      }
    });
  }

  closeEditModal(): void {
    this.showEditModal.set(false);
    this.selectedUser.set(null);
    this.editForm.reset();
    this.selectedProjectIds.set([]);
    this.editModalTab.set('details');
  }

  submitEdit(): void {
    if (this.editForm.invalid || !this.selectedUser()) return;
    const data = this.editForm.value;
    this.userService.updateUser(this.selectedUser()!.user_id, {
      name: data.name || undefined,
      role: data.role || undefined,
      is_active: data.is_active ?? undefined
    }).subscribe({
      next: () => {
        this.closeEditModal();
        this.showSuccess('User updated successfully');
        this.loadUsers();
      },
      error: (err: any) => {
        this.error.set(err.error?.message || 'Failed to update user');
      }
    });
  }

  // Tab switching
  switchEditTab(tab: 'details' | 'projects'): void {
    this.editModalTab.set(tab);
  }

  toggleProjectSelection(projectId: string): void {
    const current = this.selectedProjectIds();
    if (current.includes(projectId)) {
      this.selectedProjectIds.set(current.filter(id => id !== projectId));
    } else {
      this.selectedProjectIds.set([...current, projectId]);
    }
  }

  isProjectSelected(projectId: string): boolean {
    return this.selectedProjectIds().includes(projectId);
  }

  submitAssignProjects(): void {
    const projectIds = this.selectedProjectIds();
    if (projectIds.length === 0 || !this.selectedUser()) return;
    
    // Find a default role to use (prefer 'project_member' system role)
    const defaultRole = this.roles().find(r => r.name.toLowerCase() === 'project_member' || r.name.toLowerCase() === 'viewer') 
                        || this.roles()[0];
    
    if (!defaultRole) {
      this.error.set('No roles available. Please create a role first.');
      return;
    }
    
    // Assign to all selected projects
    const userId = this.selectedUser()!.user_id;
    let completed = 0;
    let errors: string[] = [];
    
    projectIds.forEach(projectId => {
      const request: AssignRoleRequest = {
        user_id: userId,
        project_id: projectId,
        role_id: defaultRole.role_id
      };

      this.permissionService.assignRole(request).subscribe({
        next: () => {
          completed++;
          if (completed === projectIds.length) {
            this.selectedProjectIds.set([]);
            if (errors.length === 0) {
              this.showSuccess(`User assigned to ${projectIds.length} project(s) successfully`);
            } else {
              this.showSuccess(`User assigned to ${completed - errors.length} project(s). Some failed.`);
            }
          }
        },
        error: (err: any) => {
          completed++;
          errors.push(err.error?.error || 'Failed');
          if (completed === projectIds.length) {
            this.selectedProjectIds.set([]);
            if (errors.length === projectIds.length) {
              this.error.set('Failed to assign user to projects');
            } else {
              this.showSuccess(`User assigned to ${completed - errors.length} project(s). Some failed.`);
            }
          }
        }
      });
    });
  }
  // Reset Password
  openResetPasswordModal(user: User): void {
    this.selectedUser.set(user);
    this.resetPasswordForm.reset();
    this.showResetPasswordModal.set(true);
  }

  closeResetPasswordModal(): void {
    this.showResetPasswordModal.set(false);
    this.selectedUser.set(null);
    this.resetPasswordForm.reset();
  }

  submitResetPassword(): void {
    if (this.resetPasswordForm.invalid || !this.selectedUser()) return;
    const data = this.resetPasswordForm.value;
    if (data.new_password !== data.confirm_password) {
      this.error.set('Passwords do not match');
      return;
    }
    this.userService.resetUserPassword(this.selectedUser()!.user_id, data.new_password!).subscribe({
      next: () => {
        this.closeResetPasswordModal();
      },
      error: (err: any) => {
        this.error.set(err.error?.message || 'Failed to reset password');
      }
    });
  }

  // Toggle Active
  toggleUserActive(user: User): void {
    if (user.is_active) {
      this.userService.deactivateUser(user.user_id).subscribe({
        next: () => this.loadUsers(),
        error: (err: any) => this.error.set(err.error?.message || 'Failed to deactivate user')
      });
    } else {
      this.userService.reactivateUser(user.user_id).subscribe({
        next: () => this.loadUsers(),
        error: (err: any) => this.error.set(err.error?.message || 'Failed to reactivate user')
      });
    }
  }

  // Delete
  deleteUser(user: User): void {
    if (!confirm(`Are you sure you want to permanently delete ${user.name}?`)) return;
    this.userService.deleteUser(user.user_id, true).subscribe({
      next: () => this.loadUsers(),
      error: (err: any) => this.error.set(err.error?.message || 'Failed to delete user')
    });
  }

  // Helpers
  getRoleBadgeClass(role: string): string {
    return `badge-${role}`;
  }

  getStatusClass(isActive: boolean): string {
    return isActive ? 'status-active' : 'status-inactive';
  }

  isUserAssignedToProject(projectId: string): boolean {
    return this.userMemberships().some(m => m.project_id === projectId);
  }

  removeProjectAssignment(projectId: string): void {
    const userId = this.selectedUser()?.user_id;
    if (!userId) return;

    if (!confirm('Are you sure you want to remove this project assignment?')) return;

    this.permissionService.revokeRole(projectId, userId).subscribe({
      next: () => {
        // Remove from local state
        this.userMemberships.set(this.userMemberships().filter(m => m.project_id !== projectId));
        this.selectedProjectIds.set(this.selectedProjectIds().filter(id => id !== projectId));
        this.showSuccess('Project assignment removed');
      },
      error: (err: any) => {
        this.error.set(err.error?.error || 'Failed to remove assignment');
      }
    });
  }
}
