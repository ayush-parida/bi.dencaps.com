import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, catchError, throwError } from 'rxjs';
import { environment } from '../../../environments/environment';
import { Project, CreateProjectRequest } from '../models';

@Injectable({
  providedIn: 'root'
})
export class ProjectService {
  private readonly http = inject(HttpClient);

  getProjects(): Observable<Project[]> {
    return this.http.get<Project[]>(`${environment.apiUrl}/projects`)
      .pipe(catchError(this.handleError));
  }

  getProjectById(projectId: string): Observable<Project> {
    return this.http.get<Project>(`${environment.apiUrl}/projects/${projectId}`)
      .pipe(catchError(this.handleError));
  }

  createProject(request: CreateProjectRequest): Observable<Project> {
    return this.http.post<Project>(`${environment.apiUrl}/projects`, request)
      .pipe(catchError(this.handleError));
  }

  updateProject(projectId: string, request: Partial<CreateProjectRequest>): Observable<Project> {
    return this.http.put<Project>(`${environment.apiUrl}/projects/${projectId}`, request)
      .pipe(catchError(this.handleError));
  }

  deleteProject(projectId: string): Observable<void> {
    return this.http.delete<void>(`${environment.apiUrl}/projects/${projectId}`)
      .pipe(catchError(this.handleError));
  }

  private handleError(error: any): Observable<never> {
    console.error('Project service error:', error);
    return throwError(() => error);
  }
}
