import { Component, inject, OnInit, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule, ReactiveFormsModule, FormBuilder, Validators } from '@angular/forms';
import { UserManagementService } from '../../core/services/user-management.service';
import { PermissionService } from '../../core/services/permission.service';
import { Permission } from '../../core/models/permission.model';
import { HasPermissionDirective } from '../../shared/directives/permission.directive';

interface User {
  user_id: string;
  email: string;
  name: string;
  role: string;
  is_active: boolean;
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
  private readonly fb = inject(FormBuilder);

  readonly Permission = Permission;

  users = signal<User[]>([]);
  loading = signal(false);
  error = signal<string | null>(null);
  searchQuery = signal('');

  // Modal states
  showCreateModal = signal(false);
  showEditModal = signal(false);
  showResetPasswordModal = signal(false);
  selectedUser = signal<User | null>(null);

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

  ngOnInit(): void {
    this.loadUsers();
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
    this.showEditModal.set(true);
  }

  closeEditModal(): void {
    this.showEditModal.set(false);
    this.selectedUser.set(null);
    this.editForm.reset();
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
        this.loadUsers();
      },
      error: (err: any) => {
        this.error.set(err.error?.message || 'Failed to update user');
      }
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
}
