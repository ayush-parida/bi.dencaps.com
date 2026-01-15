import { Component, inject, OnInit, ChangeDetectorRef, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormBuilder, FormGroup, Validators, ReactiveFormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { ProjectService } from '../../../core/services/project.service';
import { PermissionService } from '../../../core/services/permission.service';
import { Project, Permission } from '../../../core/models';
import { HasPermissionDirective } from '../../../shared/directives/permission.directive';

@Component({
  selector: 'app-project-list',
  imports: [CommonModule, ReactiveFormsModule, RouterLink, HasPermissionDirective],
  templateUrl: './project-list.html',
  styleUrl: './project-list.scss',
})
export class ProjectList implements OnInit {
  private readonly projectService = inject(ProjectService);
  private readonly permissionService = inject(PermissionService);
  private readonly fb = inject(FormBuilder);
  private readonly cdr = inject(ChangeDetectorRef);

  readonly Permission = Permission;
  
  projects: Project[] = [];
  isLoading: boolean = true;
  showCreateForm: boolean = false;
  createProjectForm: FormGroup;
  errorMessage: string = '';
  
  // Permissions
  canCreate = signal<boolean>(false);
  canDelete = signal<boolean>(false);

  constructor() {
    this.createProjectForm = this.fb.group({
      name: ['', [Validators.required, Validators.minLength(3)]],
      description: ['', [Validators.required]]
    });
  }

  ngOnInit(): void {
    this.loadProjects();
    this.loadPermissions();
  }

  loadPermissions(): void {
    this.permissionService.getMyPermissions().subscribe({
      next: (permissions) => {
        this.canCreate.set(permissions.is_admin || permissions.permissions.includes(Permission.ProjectCreate));
        this.canDelete.set(permissions.is_admin || permissions.permissions.includes(Permission.ProjectDelete));
      },
      error: (err) => console.error('Failed to load permissions:', err)
    });
  }

  loadProjects(): void {
    this.isLoading = true;
    console.log('Loading projects...');
    this.projectService.getProjects().subscribe({
      next: (projects) => {
        console.log('Projects loaded:', projects);
        this.projects = projects;
        this.isLoading = false;
        this.cdr.detectChanges();
      },
      error: (error) => {
        console.error('Failed to load projects:', error);
        this.isLoading = false;
        this.cdr.detectChanges();
      }
    });
  }

  toggleCreateForm(): void {
    if (!this.canCreate()) {
      return;
    }
    this.showCreateForm = !this.showCreateForm;
    if (!this.showCreateForm) {
      this.createProjectForm.reset();
      this.errorMessage = '';
    }
  }

  onSubmit(): void {
    if (this.createProjectForm.invalid || !this.canCreate()) {
      return;
    }

    this.errorMessage = '';
    this.projectService.createProject(this.createProjectForm.value).subscribe({
      next: (project) => {
        this.projects.unshift(project);
        this.toggleCreateForm();
      },
      error: (error) => {
        this.errorMessage = error.error?.error || 'Failed to create project';
      }
    });
  }

  deleteProject(project: Project): void {
    if (!this.canDelete()) {
      return;
    }
    if (!confirm(`Are you sure you want to delete "${project.name}"?`)) {
      return;
    }
    this.projectService.deleteProject(project.project_id).subscribe({
      next: () => {
        this.projects = this.projects.filter(p => p.project_id !== project.project_id);
      },
      error: (error) => {
        console.error('Failed to delete project:', error);
      }
    });
  }
}
