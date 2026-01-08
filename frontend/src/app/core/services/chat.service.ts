import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, catchError, throwError } from 'rxjs';
import { environment } from '../../../environments/environment';
import { ChatResponse, Conversation, SendMessageRequest, ApiError } from '../models';

@Injectable({
  providedIn: 'root'
})
export class ChatService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = `${environment.apiUrl}/chat`;

  sendMessage(request: SendMessageRequest): Observable<ChatResponse> {
    return this.http.post<ChatResponse>(`${this.baseUrl}/message`, request)
      .pipe(
        catchError(this.handleError)
      );
  }

  getConversation(conversationId: string): Observable<Conversation> {
    return this.http.get<Conversation>(`${this.baseUrl}/conversations/${conversationId}`)
      .pipe(
        catchError(this.handleError)
      );
  }

  getProjectConversations(projectId: string): Observable<Conversation[]> {
    return this.http.get<Conversation[]>(`${this.baseUrl}/projects/${projectId}/conversations`)
      .pipe(
        catchError(this.handleError)
      );
  }

  private handleError(error: any): Observable<never> {
    let errorMessage = 'An unknown error occurred';
    
    if (error.error instanceof ErrorEvent) {
      // Client-side error
      errorMessage = `Error: ${error.error.message}`;
    } else if (error.error?.error) {
      // Server-side error with error field
      errorMessage = error.error.error;
    } else if (error.status === 429) {
      errorMessage = 'Rate limit exceeded. Please wait before sending more messages.';
    } else if (error.status === 0) {
      errorMessage = 'Unable to connect to the server. Please check your connection.';
    } else {
      errorMessage = `Server error: ${error.status} - ${error.message}`;
    }
    
    return throwError(() => new Error(errorMessage));
  }
}
