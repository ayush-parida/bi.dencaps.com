import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { BehaviorSubject, Observable, tap, catchError, throwError } from 'rxjs';
import { Router } from '@angular/router';
import { environment } from '../../../environments/environment';
import { AuthResponse, LoginRequest, RegisterRequest, User } from '../models';
import { TokenService } from './token.service';

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private readonly http = inject(HttpClient);
  private readonly router = inject(Router);
  private readonly tokenService = inject(TokenService);
  
  private currentUserSubject = new BehaviorSubject<User | null>(null);
  public currentUser$ = this.currentUserSubject.asObservable();

  constructor() {
    this.loadUserFromToken();
  }

  register(request: RegisterRequest): Observable<AuthResponse> {
    return this.http.post<AuthResponse>(`${environment.apiUrl}/auth/register`, request)
      .pipe(
        tap(response => this.handleAuthResponse(response)),
        catchError(this.handleError)
      );
  }

  login(request: LoginRequest): Observable<AuthResponse> {
    return this.http.post<AuthResponse>(`${environment.apiUrl}/auth/login`, request)
      .pipe(
        tap(response => this.handleAuthResponse(response)),
        catchError(this.handleError)
      );
  }

  logout(): void {
    this.tokenService.clearTokens();
    this.currentUserSubject.next(null);
    this.router.navigate(['/auth/login']);
  }

  refreshToken(): Observable<{ access_token: string }> {
    const refreshToken = this.tokenService.getRefreshToken();
    if (!refreshToken) {
      return throwError(() => new Error('No refresh token available'));
    }

    return this.http.post<{ access_token: string }>(
      `${environment.apiUrl}/auth/refresh`,
      { refresh_token: refreshToken }
    ).pipe(
      tap(response => {
        this.tokenService.setAccessToken(response.access_token);
      }),
      catchError(err => {
        this.logout();
        return throwError(() => err);
      })
    );
  }

  getCurrentUser(): Observable<User> {
    return this.http.get<User>(`${environment.apiUrl}/users/me`)
      .pipe(
        tap(user => this.currentUserSubject.next(user)),
        catchError(this.handleError)
      );
  }

  getAccessToken(): string | null {
    return this.tokenService.getAccessToken();
  }

  getRefreshToken(): string | null {
    return this.tokenService.getRefreshToken();
  }

  isAuthenticated(): boolean {
    return this.tokenService.isAuthenticated();
  }

  private handleAuthResponse(response: AuthResponse): void {
    this.tokenService.setAccessToken(response.access_token);
    this.tokenService.setRefreshToken(response.refresh_token);
    this.currentUserSubject.next(response.user);
  }

  private loadUserFromToken(): void {
    if (this.isAuthenticated()) {
      this.getCurrentUser().subscribe({
        error: (err) => {
          console.warn('Failed to load user from token:', err);
          // Don't logout here - let the interceptor handle 401s
          // Just clear the current user subject
          this.currentUserSubject.next(null);
        }
      });
    }
  }

  private handleError(error: any): Observable<never> {
    console.error('Auth error:', error);
    return throwError(() => error);
  }
}
