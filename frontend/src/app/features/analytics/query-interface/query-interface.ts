import { Component, inject, OnInit, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormBuilder, FormGroup, Validators, ReactiveFormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { AnalyticsService } from '../../../core/services/analytics.service';
import { ProjectService } from '../../../core/services/project.service';
import { Project, AnalyticsQuery } from '../../../core/models';
import { ContentRendererComponent } from '../../../shared/rendering';

@Component({
  selector: 'app-query-interface',
  imports: [CommonModule, ReactiveFormsModule, RouterLink, ContentRendererComponent],
  templateUrl: './query-interface.html',
  styleUrl: './query-interface.scss',
})
export class QueryInterface implements OnInit {
  private readonly analyticsService = inject(AnalyticsService);
  private readonly projectService = inject(ProjectService);
  private readonly fb = inject(FormBuilder);
  private readonly cdr = inject(ChangeDetectorRef);

  projects: Project[] = [];
  queries: AnalyticsQuery[] = [];
  queryForm: FormGroup;
  errorMessage: string = '';
  isProcessing: boolean = false;
  currentResponse: string = '';

  constructor() {
    this.queryForm = this.fb.group({
      project_id: ['', [Validators.required]],
      query_text: ['', [Validators.required, Validators.minLength(3)]]
    });
  }

  ngOnInit(): void {
    this.loadProjects();
  }

  loadProjects(): void {
    this.projectService.getProjects().subscribe({
      next: (projects) => {
        this.projects = projects;
        if (projects.length > 0) {
          this.queryForm.patchValue({ project_id: projects[0].project_id });
          this.loadQueries(projects[0].project_id);
        }
        this.cdr.detectChanges();
      },
      error: (error) => {
        console.error('Failed to load projects:', error);
        this.cdr.detectChanges();
      }
    });
  }

  onProjectChange(event: any): void {
    const projectId = event.target.value;
    if (projectId) {
      this.loadQueries(projectId);
    }
  }

  loadQueries(projectId: string): void {
    this.analyticsService.getProjectQueries(projectId).subscribe({
      next: (queries) => {
        this.queries = queries;
        this.cdr.detectChanges();
      },
      error: (error) => {
        console.error('Failed to load queries:', error);
        this.cdr.detectChanges();
      }
    });
  }

  onSubmit(): void {
    if (this.queryForm.invalid) {
      return;
    }

    this.isProcessing = true;
    this.errorMessage = '';
    this.currentResponse = '';

    this.analyticsService.createQuery(this.queryForm.value).subscribe({
      next: (query) => {
        // Query created, now process it
        this.processQuery(query.query_id);
      },
      error: (error) => {
        this.isProcessing = false;
        this.errorMessage = error.error?.error || 'Failed to create query';
        this.cdr.detectChanges();
      }
    });
  }

  processQuery(queryId: string): void {
    this.analyticsService.processQuery(queryId).subscribe({
      next: (response) => {
        this.currentResponse = response.response;
        this.isProcessing = false;
        this.queryForm.patchValue({ query_text: '' });
        this.loadQueries(this.queryForm.value.project_id);
        this.cdr.detectChanges();
      },
      error: (error) => {
        this.isProcessing = false;
        this.errorMessage = error.error?.error || 'Failed to process query';
        this.cdr.detectChanges();
      }
    });
  }

  tryParseStructuredResponse(response: string): any {
    try {
      return JSON.parse(response);
    } catch {
      return null;
    }
  }
}
