import { Component, inject, OnInit, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule, ReactiveFormsModule, FormBuilder, Validators, FormArray } from '@angular/forms';
import { PermissionService } from '../../core/services/permission.service';
import { Role, Permission, CreateRoleRequest, UpdateRoleRequest } from '../../core/models';
import { HasPermissionDirective } from '../../shared/directives/permission.directive';

interface PermissionGroup {
  name: string;
  permissions: { value: Permission; label: string; checked: boolean }[];
}

@Component({
  selector: 'app-role-list',
  standalone: true,
  imports: [CommonModule, FormsModule, ReactiveFormsModule, HasPermissionDirective],
  templateUrl: './role-list.html',
  styleUrl: './role-list.scss',
})
export class RoleListComponent implements OnInit {
  private readonly permissionService = inject(PermissionService);
  private readonly fb = inject(FormBuilder);

  readonly Permission = Permission;

  roles = signal<Role[]>([]);
  loading = signal(false);
  error = signal<string | null>(null);
  successMessage = signal<string | null>(null);
  searchQuery = signal('');

  // Modal states
  showCreateModal = signal(false);
  showEditModal = signal(false);
  showDeleteModal = signal(false);
  selectedRole = signal<Role | null>(null);

  // Permission groups for the UI
  permissionGroups: PermissionGroup[] = [
    {
      name: 'Project Management',
      permissions: [
        { value: Permission.ProjectCreate, label: 'Create Projects', checked: false },
        { value: Permission.ProjectRead, label: 'View Projects', checked: false },
        { value: Permission.ProjectUpdate, label: 'Edit Projects', checked: false },
        { value: Permission.ProjectDelete, label: 'Delete Projects', checked: false },
        { value: Permission.ProjectManageMembers, label: 'Manage Members', checked: false },
      ]
    },
    {
      name: 'User Management',
      permissions: [
        { value: Permission.UserCreate, label: 'Create Users', checked: false },
        { value: Permission.UserRead, label: 'View Users', checked: false },
        { value: Permission.UserUpdate, label: 'Edit Users', checked: false },
        { value: Permission.UserDelete, label: 'Delete Users', checked: false },
        { value: Permission.UserManageRoles, label: 'Manage Roles', checked: false },
      ]
    },
    {
      name: 'Chat',
      permissions: [
        { value: Permission.ChatRead, label: 'View Chat History', checked: false },
        { value: Permission.ChatWrite, label: 'Send Messages', checked: false },
        { value: Permission.ChatDelete, label: 'Delete Messages', checked: false },
        { value: Permission.ChatExport, label: 'Export Chat', checked: false },
      ]
    },
    {
      name: 'Reports & Analytics',
      permissions: [
        { value: Permission.ReportCreate, label: 'Create Reports', checked: false },
        { value: Permission.ReportRead, label: 'View Reports', checked: false },
        { value: Permission.ReportExport, label: 'Export Reports', checked: false },
        { value: Permission.ReportDelete, label: 'Delete Reports', checked: false },
      ]
    },
    {
      name: 'Administration',
      permissions: [
        { value: Permission.AdminAccess, label: 'Admin Access', checked: false },
        { value: Permission.SystemSettings, label: 'System Settings', checked: false },
      ]
    }
  ];

  // Computed
  filteredRoles = computed(() => {
    const query = this.searchQuery().toLowerCase();
    if (!query) return this.roles();
    return this.roles().filter(r =>
      r.name.toLowerCase().includes(query) ||
      r.description.toLowerCase().includes(query)
    );
  });

  systemRoles = computed(() => this.roles().filter(r => r.is_system_role).length);
  customRoles = computed(() => this.roles().filter(r => !r.is_system_role).length);

  // Forms
  createForm = this.fb.group({
    name: ['', [Validators.required, Validators.minLength(2), Validators.maxLength(50)]],
    description: ['', [Validators.maxLength(200)]]
  });

  editForm = this.fb.group({
    name: ['', [Validators.required, Validators.minLength(2), Validators.maxLength(50)]],
    description: ['', [Validators.maxLength(200)]]
  });

  // Track selected permissions for create/edit
  selectedPermissions = signal<Set<string>>(new Set());

  ngOnInit(): void {
    this.loadRoles();
  }

  loadRoles(): void {
    this.loading.set(true);
    this.error.set(null);
    this.permissionService.getRoles().subscribe({
      next: (roles: Role[]) => {
        this.roles.set(roles);
        this.loading.set(false);
      },
      error: (err: any) => {
        this.error.set(err.error?.error || 'Failed to load roles');
        this.loading.set(false);
      }
    });
  }

  // Permission handling
  togglePermission(permission: Permission): void {
    const perms = new Set(this.selectedPermissions());
    if (perms.has(permission)) {
      perms.delete(permission);
    } else {
      perms.add(permission);
    }
    this.selectedPermissions.set(perms);
  }

  isPermissionSelected(permission: Permission): boolean {
    return this.selectedPermissions().has(permission);
  }

  selectAllInGroup(group: PermissionGroup): void {
    const perms = new Set(this.selectedPermissions());
    group.permissions.forEach(p => perms.add(p.value));
    this.selectedPermissions.set(perms);
  }

  deselectAllInGroup(group: PermissionGroup): void {
    const perms = new Set(this.selectedPermissions());
    group.permissions.forEach(p => perms.delete(p.value));
    this.selectedPermissions.set(perms);
  }

  isGroupFullySelected(group: PermissionGroup): boolean {
    return group.permissions.every(p => this.selectedPermissions().has(p.value));
  }

  isGroupPartiallySelected(group: PermissionGroup): boolean {
    const selected = group.permissions.filter(p => this.selectedPermissions().has(p.value));
    return selected.length > 0 && selected.length < group.permissions.length;
  }

  toggleGroup(group: PermissionGroup): void {
    if (this.isGroupFullySelected(group)) {
      this.deselectAllInGroup(group);
    } else {
      this.selectAllInGroup(group);
    }
  }

  // Create
  openCreateModal(): void {
    this.createForm.reset();
    this.selectedPermissions.set(new Set());
    this.showCreateModal.set(true);
  }

  closeCreateModal(): void {
    this.showCreateModal.set(false);
  }

  submitCreate(): void {
    if (this.createForm.invalid) return;

    const request: CreateRoleRequest = {
      name: this.createForm.value.name!,
      description: this.createForm.value.description || '',
      permissions: Array.from(this.selectedPermissions())
    };

    this.loading.set(true);
    this.permissionService.createRole(request).subscribe({
      next: () => {
        this.showSuccessMessage('Role created successfully');
        this.closeCreateModal();
        this.loadRoles();
      },
      error: (err: any) => {
        this.error.set(err.error?.error || 'Failed to create role');
        this.loading.set(false);
      }
    });
  }

  // Edit
  openEditModal(role: Role): void {
    this.selectedRole.set(role);
    this.editForm.patchValue({
      name: role.name,
      description: role.description
    });
    this.selectedPermissions.set(new Set(role.permissions));
    this.showEditModal.set(true);
  }

  closeEditModal(): void {
    this.showEditModal.set(false);
    this.selectedRole.set(null);
  }

  submitEdit(): void {
    if (this.editForm.invalid || !this.selectedRole()) return;

    const request: UpdateRoleRequest = {
      name: this.editForm.value.name!,
      description: this.editForm.value.description || '',
      permissions: Array.from(this.selectedPermissions())
    };

    this.loading.set(true);
    this.permissionService.updateRole(this.selectedRole()!.role_id, request).subscribe({
      next: () => {
        this.showSuccessMessage('Role updated successfully');
        this.closeEditModal();
        this.loadRoles();
      },
      error: (err: any) => {
        this.error.set(err.error?.error || 'Failed to update role');
        this.loading.set(false);
      }
    });
  }

  // Delete
  openDeleteModal(role: Role): void {
    this.selectedRole.set(role);
    this.showDeleteModal.set(true);
  }

  closeDeleteModal(): void {
    this.showDeleteModal.set(false);
    this.selectedRole.set(null);
  }

  confirmDelete(): void {
    if (!this.selectedRole()) return;

    this.loading.set(true);
    this.permissionService.deleteRole(this.selectedRole()!.role_id).subscribe({
      next: () => {
        this.showSuccessMessage('Role deleted successfully');
        this.closeDeleteModal();
        this.loadRoles();
      },
      error: (err: any) => {
        this.error.set(err.error?.error || 'Failed to delete role');
        this.loading.set(false);
      }
    });
  }

  // Utilities
  showSuccessMessage(message: string): void {
    this.successMessage.set(message);
    setTimeout(() => this.successMessage.set(null), 3000);
  }

  dismissError(): void {
    this.error.set(null);
  }

  dismissSuccess(): void {
    this.successMessage.set(null);
  }

  getPermissionCount(role: Role): number {
    return role.permissions.length;
  }

  formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    });
  }
}
