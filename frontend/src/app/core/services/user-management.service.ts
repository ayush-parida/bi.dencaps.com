import { Injectable, inject } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../../../environments/environment';

export interface User {
  user_id: string;
  email: string;
  name: string;
  role: string;
  tenant_id: string;
  is_active: boolean;
  created_at: string;
  updated_at?: string;
}

export interface CreateUserRequest {
  email: string;
  name: string;
  password: string;
  role: string;
}

export interface UpdateUserRequest {
  name?: string;
  role?: string;
  is_active?: boolean;
}

export interface ChangePasswordRequest {
  current_password: string;
  new_password: string;
}

@Injectable({
  providedIn: 'root'
})
export class UserManagementService {
  private readonly http = inject(HttpClient);
  private readonly apiUrl = `${environment.apiUrl}/users`;

  /**
   * Get all users in the tenant
   */
  getUsers(): Observable<User[]> {
    return this.http.get<User[]>(this.apiUrl);
  }

  /**
   * Search users by query
   */
  searchUsers(query: string): Observable<User[]> {
    const params = new HttpParams().set('q', query);
    return this.http.get<User[]>(`${this.apiUrl}/search`, { params });
  }

  /**
   * Get a specific user by ID
   */
  getUser(userId: string): Observable<User> {
    return this.http.get<User>(`${this.apiUrl}/${userId}`);
  }

  /**
   * Get current user profile
   */
  getCurrentUser(): Observable<User> {
    return this.http.get<User>(`${this.apiUrl}/me`);
  }

  /**
   * Create a new user (admin only)
   */
  createUser(request: CreateUserRequest): Observable<User> {
    return this.http.post<User>(this.apiUrl, request);
  }

  /**
   * Update a user
   */
  updateUser(userId: string, request: UpdateUserRequest): Observable<User> {
    return this.http.put<User>(`${this.apiUrl}/${userId}`, request);
  }

  /**
   * Change own password
   */
  changePassword(request: ChangePasswordRequest): Observable<{ message: string }> {
    return this.http.put<{ message: string }>(`${this.apiUrl}/me/password`, request);
  }

  /**
   * Reset a user's password (admin only)
   */
  resetUserPassword(userId: string, newPassword: string): Observable<{ message: string }> {
    return this.http.put<{ message: string }>(`${this.apiUrl}/${userId}/password`, {
      new_password: newPassword
    });
  }

  /**
   * Delete a user
   */
  deleteUser(userId: string, permanent: boolean = false): Observable<{ message: string }> {
    const params = new HttpParams().set('permanent', permanent.toString());
    return this.http.delete<{ message: string }>(`${this.apiUrl}/${userId}`, { params });
  }

  /**
   * Deactivate a user (soft delete)
   */
  deactivateUser(userId: string): Observable<User> {
    return this.updateUser(userId, { is_active: false });
  }

  /**
   * Reactivate a user
   */
  reactivateUser(userId: string): Observable<User> {
    return this.updateUser(userId, { is_active: true });
  }

  /**
   * Change a user's role (admin only)
   */
  changeUserRole(userId: string, newRole: string): Observable<User> {
    return this.updateUser(userId, { role: newRole });
  }
}
