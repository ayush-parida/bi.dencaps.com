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
        // Clean and trim the final content before saving/completing
        const cleanedContent = this.cleanFinalContent(fullContent);
        // Save the response to the database before completing
        if (conversationId && cleanedContent) {
          this.saveStreamedResponse(conversationId, cleanedContent).subscribe({
            next: () => console.log('Streamed response saved'),
            error: (err) => console.error('Failed to save streamed response:', err)
          });
        }
        subject.next({ conversationId, content: cleanedContent, done: true });
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
                  // Accumulate content chunks, filtering out [DONE] marker
                  const cleanChunk = this.cleanContent(parsed.content);
                  if (cleanChunk) {
                    fullContent += cleanChunk;
                    subject.next({ conversationId, content: this.cleanContent(fullContent), done: false });
                  }
                } else if (parsed.token !== undefined) {
                  // Some APIs send token instead of content
                  const cleanChunk = this.cleanContent(parsed.token);
                  if (cleanChunk) {
                    fullContent += cleanChunk;
                    subject.next({ conversationId, content: this.cleanContent(fullContent), done: false });
                  }
                } else if (typeof parsed === 'string') {
                  const cleanChunk = this.cleanContent(parsed);
                  if (cleanChunk) {
                    fullContent += cleanChunk;
                    subject.next({ conversationId, content: this.cleanContent(fullContent), done: false });
                  }
                }
              } catch {
                // If not JSON, treat as raw content (for RAG API format)
                if (data && data !== '{}') {
                  const cleanChunk = this.cleanContent(data);
                  if (cleanChunk) {
                    fullContent += cleanChunk;
                    subject.next({ conversationId, content: this.cleanContent(fullContent), done: false });
                  }
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

  /**
   * Regenerate a response from a specific message index
   * This removes messages from that index onwards and regenerates
   */
  regenerateMessageStream(conversationId: string, fromIndex: number): Observable<StreamingChatResponse> {
    const subject = new Subject<StreamingChatResponse>();
    
    const token = localStorage.getItem('access_token');
    
    fetch(`${this.baseUrl}/message/regenerate`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`
      },
      body: JSON.stringify({
        conversation_id: conversationId,
        from_index: fromIndex
      })
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
      let convId = conversationId;
      let fullContent = '';

      const finishStream = () => {
        // Clean and trim the final content
        const cleanedContent = this.cleanFinalContent(fullContent);
        // Save the regenerated response
        if (convId && cleanedContent) {
          this.saveStreamedResponse(convId, cleanedContent).subscribe({
            next: () => console.log('Regenerated response saved'),
            error: (err) => console.error('Failed to save regenerated response:', err)
          });
        }
        subject.next({ conversationId: convId, content: cleanedContent, done: true });
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
          
          const lines = buffer.split('\n');
          buffer = lines.pop() || '';

          for (const line of lines) {
            if (line.startsWith('event: ')) {
              const eventType = line.substring(7).trim();
              if (eventType === 'done') {
                finishStream();
                return;
              }
            } else if (line.startsWith('data: ')) {
              const data = line.substring(6);
              try {
                const parsed = JSON.parse(data);
                
                if (parsed.conversation_id) {
                  convId = parsed.conversation_id;
                } else if (parsed.error) {
                  subject.error(new Error(parsed.error));
                  return;
                } else if (parsed.content !== undefined) {
                  const cleanChunk = this.cleanContent(parsed.content);
                  if (cleanChunk) {
                    fullContent += cleanChunk;
                    subject.next({ conversationId: convId, content: this.cleanContent(fullContent), done: false });
                  }
                } else if (parsed.token !== undefined) {
                  const cleanChunk = this.cleanContent(parsed.token);
                  if (cleanChunk) {
                    fullContent += cleanChunk;
                    subject.next({ conversationId: convId, content: this.cleanContent(fullContent), done: false });
                  }
                } else if (typeof parsed === 'string') {
                  const cleanChunk = this.cleanContent(parsed);
                  if (cleanChunk) {
                    fullContent += cleanChunk;
                    subject.next({ conversationId: convId, content: this.cleanContent(fullContent), done: false });
                  }
                }
              } catch {
                if (data && data !== '{}') {
                  const cleanChunk = this.cleanContent(data);
                  if (cleanChunk) {
                    fullContent += cleanChunk;
                    subject.next({ conversationId: convId, content: this.cleanContent(fullContent), done: false });
                  }
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

  /**
   * Clean content by removing [DONE] markers and other artifacts
   * Does NOT trim to preserve spaces between streaming chunks
   */
  private cleanContent(content: string): string {
    if (!content) return '';
    return content
      .replace(/\[DONE\]/g, '')  // Remove [DONE] marker
      .replace(/data:\s*\[DONE\]/g, '');  // Remove SSE formatted [DONE]
  }

  /**
   * Clean and trim final content (for saving and display)
   */
  private cleanFinalContent(content: string): string {
    return this.cleanContent(content).trim();
  }
}
