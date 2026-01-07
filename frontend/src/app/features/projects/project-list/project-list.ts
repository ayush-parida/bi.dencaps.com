import { Component, inject, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormBuilder, FormGroup, Validators, ReactiveFormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { ProjectService } from '../../../core/services/project.service';
import { AuthService } from '../../../core/services/auth.service';
import { Project } from '../../../core/models';

@Component({
  selector: 'app-project-list',
  imports: [CommonModule, ReactiveFormsModule, RouterLink],
  templateUrl: './project-list.html',
  styleUrl: './project-list.scss',
})
export class ProjectList implements OnInit {
  private readonly projectService = inject(ProjectService);
  private readonly authService = inject(AuthService);
  private readonly fb = inject(FormBuilder);

  projects: Project[] = [];
  isLoading: boolean = true;
  showCreateForm: boolean = false;
  createProjectForm: FormGroup;
  errorMessage: string = '';

  constructor() {
    this.createProjectForm = this.fb.group({
      name: ['', [Validators.required, Validators.minLength(3)]],
      description: ['', [Validators.required]]
    });
  }

  ngOnInit(): void {
    this.loadProjects();
  }

  loadProjects(): void {
    this.isLoading = true;
    this.projectService.getProjects().subscribe({
      next: (projects) => {
        this.projects = projects;
        this.isLoading = false;
      },
      error: (error) => {
        console.error('Failed to load projects:', error);
        this.isLoading = false;
      }
    });
  }

  toggleCreateForm(): void {
    this.showCreateForm = !this.showCreateForm;
    if (!this.showCreateForm) {
      this.createProjectForm.reset();
      this.errorMessage = '';
    }
  }

  onSubmit(): void {
    if (this.createProjectForm.invalid) {
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

  logout(): void {
    this.authService.logout();
  }
}
