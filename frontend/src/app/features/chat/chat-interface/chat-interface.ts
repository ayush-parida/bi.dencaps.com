import { Component, OnInit, OnDestroy, inject, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { Subject, takeUntil } from 'rxjs';
import { ChatService } from '../../../core/services/chat.service';
import { ProjectService } from '../../../core/services/project.service';
import { ChatMessage, Project } from '../../../core/models';
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
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);
  private readonly destroy$ = new Subject<void>();

  projectId = signal<string | null>(null);
  project = signal<Project | null>(null);
  projects = signal<Project[]>([]);
  currentConversationId = signal<string | null>(null);
  messages = signal<ChatMessage[]>([]);
  
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
    
    if (projectId) {
      this.loadProject(projectId);
      this.router.navigate(['/chat'], { 
        queryParams: { projectId } 
      });
    } else {
      this.project.set(null);
      this.router.navigate(['/chat']);
    }
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
          this.messages.set(conversation.messages);
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

    this.chatService.sendMessage({
      message,
      project_id: pid,
      conversation_id: this.currentConversationId() || undefined
    }).pipe(takeUntil(this.destroy$))
      .subscribe({
        next: (response) => {
          // Update conversation ID if it's a new conversation
          if (!this.currentConversationId()) {
            this.currentConversationId.set(response.conversation_id);
            // Update URL with new conversation ID
            this.router.navigate([], {
              queryParams: { 
                projectId: pid, 
                conversationId: response.conversation_id 
              },
              queryParamsHandling: 'merge'
            });
          }

          // Add AI response to messages
          this.messages.update(msgs => [...msgs, response.message]);
          this.isSending.set(false);
        },
        error: (err) => {
          // Remove the optimistically added user message on error
          this.messages.update(msgs => msgs.slice(0, -1));
          this.messageInput = message; // Restore the message
          this.error.set(err.message);
          this.isSending.set(false);
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
      return JSON.parse(content);
    } catch {
      return null;
    }
  }
}
