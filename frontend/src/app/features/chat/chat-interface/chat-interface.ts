import { Component, OnInit, OnDestroy, inject, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { Subject, takeUntil } from 'rxjs';
import { ChatService, StreamingChatResponse } from '../../../core/services/chat.service';
import { ProjectService } from '../../../core/services/project.service';
import { PermissionService } from '../../../core/services/permission.service';
import { ChatMessage, Project, Permission } from '../../../core/models';
import { ContentRendererComponent } from '../../../shared/rendering';
import { MarkdownRendererComponent } from '../../../shared/markdown-renderer/markdown-renderer.component';

@Component({
  selector: 'app-chat-interface',
  standalone: true,
  imports: [CommonModule, FormsModule, ContentRendererComponent, MarkdownRendererComponent],
  templateUrl: './chat-interface.html',
  styleUrls: ['./chat-interface.scss']
})
export class ChatInterfaceComponent implements OnInit, OnDestroy {
  private readonly chatService = inject(ChatService);
  private readonly projectService = inject(ProjectService);
  private readonly permissionService = inject(PermissionService);
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);
  private readonly destroy$ = new Subject<void>();

  projectId = signal<string | null>(null);
  project = signal<Project | null>(null);
  projects = signal<Project[]>([]);
  currentConversationId = signal<string | null>(null);
  messages = signal<ChatMessage[]>([]);
  
  // Permission signals
  canRead = signal<boolean>(false);
  canWrite = signal<boolean>(false);
  canExport = signal<boolean>(false);
  canDelete = signal<boolean>(false);
  permissionsLoaded = signal<boolean>(false);
  
  // Computed chat title from first user message
  chatTitle = computed(() => {
    const msgs = this.messages();
    const firstUserMsg = msgs.find(m => m.role === 'user');
    if (firstUserMsg) {
      const title = firstUserMsg.content.substring(0, 60);
      return title.length < firstUserMsg.content.length ? title + '...' : title;
    }
    return 'New Chat';
  });
  
  messageInput = '';
  isLoading = signal<boolean>(false);
  isSending = signal<boolean>(false);
  isStreaming = signal<boolean>(false);
  streamingContent = signal<string>('');
  error = signal<string | null>(null);

  ngOnInit(): void {
    // Load all projects
    this.loadProjects();
    
    // Listen to route changes for project and conversation selection
    this.route.queryParams.pipe(takeUntil(this.destroy$)).subscribe(params => {
      const pid = params['projectId'];
      const convId = params['conversationId'];
      
      if (pid && pid !== this.projectId()) {
        this.projectId.set(pid);
        this.loadProject(pid);
        this.loadProjectPermissions(pid); // Load permissions when project changes
      }
      
      if (convId && convId !== this.currentConversationId()) {
        this.currentConversationId.set(convId);
        this.loadConversation(convId);
      } else if (!convId && this.currentConversationId()) {
        // New chat - clear messages
        this.currentConversationId.set(null);
        this.messages.set([]);
      }
      
      if (!pid) {
        this.error.set(null); // Don't show error, just show project selector
      }
    });
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  loadProjects(): void {
    this.projectService.getProjects()
      .pipe(takeUntil(this.destroy$))
      .subscribe({
        next: (projects) => {
          this.projects.set(projects);
        },
        error: (err) => {
          console.error('Failed to load projects:', err);
        }
      });
  }

  onProjectChange(projectId: string): void {
    this.projectId.set(projectId);
    this.currentConversationId.set(null);
    this.messages.set([]);
    this.error.set(null);
    
    // Reset permissions
    this.canRead.set(false);
    this.canWrite.set(false);
    this.canExport.set(false);
    this.canDelete.set(false);
    this.permissionsLoaded.set(false);
    
    if (projectId) {
      this.loadProject(projectId);
      this.loadProjectPermissions(projectId);
      this.router.navigate(['/chat'], { 
        queryParams: { projectId } 
      });
    } else {
      this.project.set(null);
      this.router.navigate(['/chat']);
    }
  }

  loadProjectPermissions(projectId: string): void {
    this.permissionService.getMyPermissions(projectId)
      .pipe(takeUntil(this.destroy$))
      .subscribe({
        next: (permissions) => {
          this.canRead.set(permissions.is_admin || permissions.permissions.includes(Permission.ChatRead));
          this.canWrite.set(permissions.is_admin || permissions.permissions.includes(Permission.ChatWrite));
          this.canExport.set(permissions.is_admin || permissions.permissions.includes(Permission.ChatExport));
          this.canDelete.set(permissions.is_admin || permissions.permissions.includes(Permission.ChatDelete));
          this.permissionsLoaded.set(true);
        },
        error: (err) => {
          console.error('Failed to load permissions:', err);
          this.permissionsLoaded.set(true);
        }
      });
  }

  loadProject(projectId: string): void {
    this.projectService.getProjectById(projectId)
      .pipe(takeUntil(this.destroy$))
      .subscribe({
        next: (project) => {
          this.project.set(project);
          this.error.set(null);
        },
        error: (err) => {
          this.error.set(`Failed to load project: ${err.message}`);
        }
      });
  }

  loadConversation(conversationId: string): void {
    this.isLoading.set(true);
    this.chatService.getConversation(conversationId)
      .pipe(takeUntil(this.destroy$))
      .subscribe({
        next: (conversation) => {
          // Clean [DONE] markers from loaded messages
          const cleanedMessages = conversation.messages.map(msg => ({
            ...msg,
            content: msg.content
              .replace(/\[DONE\]/g, '')
              .replace(/data:\s*\[DONE\]/g, '')
              .trim()
          }));
          this.messages.set(cleanedMessages);
          this.error.set(null);
          this.isLoading.set(false);
        },
        error: (err) => {
          this.error.set(`Failed to load conversation: ${err.message}`);
          this.isLoading.set(false);
        }
      });
  }

  sendMessage(): void {
    const message = this.messageInput.trim();
    const pid = this.projectId();
    
    if (!message || !pid) {
      return;
    }

    // Check permission before sending
    if (!this.canWrite()) {
      this.error.set('You do not have permission to send messages in this project.');
      return;
    }

    this.isSending.set(true);
    this.error.set(null);

    // Add user message to UI immediately
    const userMessage: ChatMessage = {
      role: 'user',
      content: message,
      timestamp: new Date().toISOString()
    };
    this.messages.update(msgs => [...msgs, userMessage]);
    this.messageInput = '';

    // Add placeholder for streaming assistant response
    const streamingMessage: ChatMessage = {
      role: 'assistant',
      content: '',
      timestamp: new Date().toISOString()
    };
    this.messages.update(msgs => [...msgs, streamingMessage]);
    this.isStreaming.set(true);

    this.chatService.sendMessageStream({
      message,
      project_id: pid,
      conversation_id: this.currentConversationId() || undefined
    }).pipe(takeUntil(this.destroy$))
      .subscribe({
        next: (response: StreamingChatResponse) => {
          // Update conversation ID if it's a new conversation
          if (response.conversationId && !this.currentConversationId()) {
            this.currentConversationId.set(response.conversationId);
            // Update URL with new conversation ID
            this.router.navigate([], {
              queryParams: { 
                projectId: pid, 
                conversationId: response.conversationId 
              },
              queryParamsHandling: 'merge'
            });
          }

          // Update the streaming message content
          this.messages.update(msgs => {
            const updated = [...msgs];
            const lastMsg = updated[updated.length - 1];
            if (lastMsg && lastMsg.role === 'assistant') {
              lastMsg.content = response.content;
            }
            return updated;
          });
          this.streamingContent.set(response.content);
        },
        error: (err) => {
          // Remove the placeholder messages on error
          this.messages.update(msgs => msgs.slice(0, -2));
          this.messageInput = message; // Restore the message
          this.error.set(err.message);
          this.isSending.set(false);
          this.isStreaming.set(false);
          this.streamingContent.set('');
        },
        complete: () => {
          this.isSending.set(false);
          this.isStreaming.set(false);
          this.streamingContent.set('');
        }
      });
  }

  onKeyPress(event: KeyboardEvent): void {
    // Send on Enter, but allow Shift+Enter for new lines
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      this.sendMessage();
    }
  }

  formatTimestamp(timestamp: string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) {
      return `${days}d ago`;
    } else if (hours > 0) {
      return `${hours}h ago`;
    } else if (minutes > 0) {
      return `${minutes}m ago`;
    } else {
      return 'Just now';
    }
  }

  trackByIndex(index: number): number {
    return index;
  }

  isStructuredContent(message: ChatMessage): boolean {
    return !!message.structured_content;
  }

  tryParseStructuredContent(content: string): any {
    try {
      const parsed = JSON.parse(content);
      // Only return if it's a valid structured response (has items array)
      if (parsed && typeof parsed === 'object' && Array.isArray(parsed.items) && parsed.items.length > 0) {
        return parsed;
      }
      return null;
    } catch {
      return null;
    }
  }

  /**
   * Regenerate an AI response at a specific index by resending the preceding user message
   * Uses the regenerate endpoint which properly truncates and regenerates
   */
  regenerateResponse(assistantMsgIndex: number): void {
    const msgs = this.messages();
    const convId = this.currentConversationId();
    
    if (!convId || assistantMsgIndex <= 0) {
      this.error.set('Cannot regenerate: No active conversation');
      return;
    }

    // Find the user message that preceded this assistant message
    let userMsgIndex = -1;
    for (let i = assistantMsgIndex - 1; i >= 0; i--) {
      if (msgs[i].role === 'user') {
        userMsgIndex = i;
        break;
      }
    }
    
    if (userMsgIndex === -1) {
      this.error.set('Cannot regenerate: No user message found');
      return;
    }

    // Update UI immediately - remove messages from the assistant onwards
    this.messages.update(msgList => {
      return msgList.slice(0, userMsgIndex + 1);
    });

    // Set sending state
    this.isSending.set(true);
    this.error.set(null);

    // Add placeholder for streaming assistant response
    const streamingMessage: ChatMessage = {
      role: 'assistant',
      content: '',
      timestamp: new Date().toISOString()
    };
    this.messages.update(msgsList => [...msgsList, streamingMessage]);
    this.isStreaming.set(true);

    // Use the regenerate endpoint with the original assistant message index
    this.chatService.regenerateMessageStream(convId, assistantMsgIndex)
      .pipe(takeUntil(this.destroy$))
      .subscribe({
        next: (response: StreamingChatResponse) => {
          // Update the streaming message content
          this.messages.update(allMsgs => {
            const updated = [...allMsgs];
            const lastMsg = updated[updated.length - 1];
            if (lastMsg && lastMsg.role === 'assistant') {
              lastMsg.content = response.content;
            }
            return updated;
          });
          this.streamingContent.set(response.content);
        },
        error: (err) => {
          // Remove the placeholder on error
          this.messages.update(allMsgs => allMsgs.slice(0, -1));
          this.error.set(err.message);
          this.isSending.set(false);
          this.isStreaming.set(false);
          this.streamingContent.set('');
        },
        complete: () => {
          this.isSending.set(false);
          this.isStreaming.set(false);
          this.streamingContent.set('');
        }
      });
  }

  /**
   * Check if the given message can be regenerated (any assistant message when not streaming)
   */
  canRegenerateMessage(message: ChatMessage, index: number): boolean {
    // Show regenerate for any assistant message when not sending
    return message.role === 'assistant' && 
           !this.isSending() && 
           !this.isStreaming();
  }
}
