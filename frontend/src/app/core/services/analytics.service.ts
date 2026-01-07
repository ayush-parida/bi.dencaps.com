import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, catchError, throwError } from 'rxjs';
import { environment } from '../../../environments/environment';
import { AnalyticsQuery, CreateQueryRequest } from '../models';

@Injectable({
  providedIn: 'root'
})
export class AnalyticsService {
  private readonly http = inject(HttpClient);

  createQuery(request: CreateQueryRequest): Observable<AnalyticsQuery> {
    return this.http.post<AnalyticsQuery>(`${environment.apiUrl}/analytics/queries`, request)
      .pipe(catchError(this.handleError));
  }

  processQuery(queryId: string): Observable<{ query_id: string; response: string }> {
    return this.http.post<{ query_id: string; response: string }>(
      `${environment.apiUrl}/analytics/queries/${queryId}/process`,
      {}
    ).pipe(catchError(this.handleError));
  }

  getQueryById(queryId: string): Observable<AnalyticsQuery> {
    return this.http.get<AnalyticsQuery>(`${environment.apiUrl}/analytics/queries/${queryId}`)
      .pipe(catchError(this.handleError));
  }

  getProjectQueries(projectId: string): Observable<AnalyticsQuery[]> {
    return this.http.get<AnalyticsQuery[]>(
      `${environment.apiUrl}/analytics/projects/${projectId}/queries`
    ).pipe(catchError(this.handleError));
  }

  private handleError(error: any): Observable<never> {
    console.error('Analytics service error:', error);
    return throwError(() => error);
  }
}
