import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, catchError, throwError, Subject } from 'rxjs';
import { environment } from '../../../environments/environment';
import { ChatResponse, Conversation, ConversationSummary, SendMessageRequest, ApiError } from '../models';

export interface StreamingChatResponse {
  conversationId: string;
  content: string;
  done: boolean;
  error?: string;
}

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

  /**
   * Send a message and receive a streaming response via SSE
   * Returns a Subject that emits partial content as it arrives
   */
  sendMessageStream(request: SendMessageRequest): Observable<StreamingChatResponse> {
    const subject = new Subject<StreamingChatResponse>();
    
    // Get the auth token from localStorage
    const token = localStorage.getItem('access_token');
    
    // Create fetch request for SSE (HttpClient doesn't support streaming well)
    fetch(`${this.baseUrl}/message/stream`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`
      },
      body: JSON.stringify(request)
    }).then(async response => {
      if (!response.ok) {
        const errorData = await response.json().catch(() => ({ error: 'Unknown error' }));
        subject.error(new Error(errorData.error || `HTTP ${response.status}`));
        return;
      }

      const reader = response.body?.getReader();
      if (!reader) {
        subject.error(new Error('No response body'));
        return;
      }

      const decoder = new TextDecoder();
      let buffer = '';
      let conversationId = '';
      let fullContent = '';

      const finishStream = () => {
        // Save the response to the database before completing
        if (conversationId && fullContent) {
          this.saveStreamedResponse(conversationId, fullContent).subscribe({
            next: () => console.log('Streamed response saved'),
            error: (err) => console.error('Failed to save streamed response:', err)
          });
        }
        subject.next({ conversationId, content: fullContent, done: true });
        subject.complete();
      };

      try {
        while (true) {
          const { done, value } = await reader.read();
          
          if (done) {
            finishStream();
            break;
          }

          buffer += decoder.decode(value, { stream: true });
          
          // Parse SSE events from buffer
          const lines = buffer.split('\n');
          buffer = lines.pop() || ''; // Keep incomplete line in buffer

          for (const line of lines) {
            if (line.startsWith('event: ')) {
              const eventType = line.substring(7).trim();
              
              if (eventType === 'done') {
                finishStream();
                return;
              } else if (eventType === 'error') {
                // Error will be in next data line
                continue;
              } else if (eventType === 'init') {
                // Init event contains conversation_id
                continue;
              }
            } else if (line.startsWith('data: ')) {
              const data = line.substring(6);
              try {
                const parsed = JSON.parse(data);
                
                if (parsed.conversation_id) {
                  conversationId = parsed.conversation_id;
                } else if (parsed.error) {
                  subject.error(new Error(parsed.error));
                  return;
                } else if (parsed.content !== undefined) {
                  // Accumulate content chunks
                  fullContent += parsed.content;
                  subject.next({ conversationId, content: fullContent, done: false });
                } else if (parsed.token !== undefined) {
                  // Some APIs send token instead of content
                  fullContent += parsed.token;
                  subject.next({ conversationId, content: fullContent, done: false });
                } else if (typeof parsed === 'string') {
                  fullContent += parsed;
                  subject.next({ conversationId, content: fullContent, done: false });
                }
              } catch {
                // If not JSON, treat as raw content (for RAG API format)
                if (data && data !== '{}') {
                  fullContent += data;
                  subject.next({ conversationId, content: fullContent, done: false });
                }
              }
            }
          }
        }
      } catch (error) {
        subject.error(error);
      }
    }).catch(error => {
      subject.error(error);
    });

    return subject.asObservable();
  }

  /**
   * Save the streamed assistant response to the database
   * Called after streaming is complete
   */
  saveStreamedResponse(conversationId: string, content: string): Observable<{ success: boolean }> {
    return this.http.post<{ success: boolean }>(`${this.baseUrl}/message/stream/save`, {
      conversation_id: conversationId,
      content
    }).pipe(
      catchError(this.handleError)
    );
  }

  getConversation(conversationId: string): Observable<Conversation> {
    return this.http.get<Conversation>(`${this.baseUrl}/conversations/${conversationId}`)
      .pipe(
        catchError(this.handleError)
      );
  }

  deleteConversation(conversationId: string): Observable<{ success: boolean }> {
    return this.http.delete<{ success: boolean }>(`${this.baseUrl}/conversations/${conversationId}`)
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

  /** Get lightweight conversation summaries for sidebar (without full messages) */
  getProjectConversationSummaries(projectId: string): Observable<ConversationSummary[]> {
    return this.http.get<ConversationSummary[]>(`${this.baseUrl}/projects/${projectId}/conversations/summaries`)
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
