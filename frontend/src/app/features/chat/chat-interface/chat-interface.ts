import { Component, OnInit, OnDestroy, inject, signal, effect } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute } from '@angular/router';
import { Subject, takeUntil } from 'rxjs';
import { ChatService } from '../../../core/services/chat.service';
import { ProjectService } from '../../../core/services/project.service';
import { ChatMessage, Conversation, Project } from '../../../core/models';

@Component({
  selector: 'app-chat-interface',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './chat-interface.html',
  styleUrls: ['./chat-interface.scss']
})
export class ChatInterfaceComponent implements OnInit, OnDestroy {
  private readonly chatService = inject(ChatService);
  private readonly projectService = inject(ProjectService);
  private readonly route = inject(ActivatedRoute);
  private readonly destroy$ = new Subject<void>();

  projectId = signal<string | null>(null);
  project = signal<Project | null>(null);
  currentConversationId = signal<string | null>(null);
  conversations = signal<Conversation[]>([]);
  messages = signal<ChatMessage[]>([]);
  messageInput = signal<string>('');
  isLoading = signal<boolean>(false);
  isSending = signal<boolean>(false);
  error = signal<string | null>(null);

  constructor() {
    // Effect to load conversations when project changes
    effect(() => {
      const pid = this.projectId();
      if (pid) {
        this.loadConversations(pid);
      }
    });
  }

  ngOnInit(): void {
    // Get project ID from route
    this.route.queryParams.pipe(takeUntil(this.destroy$)).subscribe(params => {
      const pid = params['projectId'];
      if (pid) {
        this.projectId.set(pid);
        this.loadProject(pid);
      } else {
        this.error.set('No project ID provided');
      }
    });
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  loadProject(projectId: string): void {
    this.projectService.getProjectById(projectId)
      .pipe(takeUntil(this.destroy$))
      .subscribe({
        next: (project) => {
          this.project.set(project);
        },
        error: (err) => {
          this.error.set(`Failed to load project: ${err.message}`);
        }
      });
  }

  loadConversations(projectId: string): void {
    this.isLoading.set(true);
    this.chatService.getProjectConversations(projectId)
      .pipe(takeUntil(this.destroy$))
      .subscribe({
        next: (conversations) => {
          this.conversations.set(conversations);
          this.isLoading.set(false);
          
          // If there are conversations, load the most recent one
          if (conversations.length > 0) {
            const mostRecent = conversations.sort((a, b) => 
              new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
            )[0];
            this.loadConversation(mostRecent.conversation_id);
          }
        },
        error: (err) => {
          this.error.set(`Failed to load conversations: ${err.message}`);
          this.isLoading.set(false);
        }
      });
  }

  loadConversation(conversationId: string): void {
    this.currentConversationId.set(conversationId);
    this.chatService.getConversation(conversationId)
      .pipe(takeUntil(this.destroy$))
      .subscribe({
        next: (conversation) => {
          this.messages.set(conversation.messages);
          this.error.set(null);
        },
        error: (err) => {
          this.error.set(`Failed to load conversation: ${err.message}`);
        }
      });
  }

  startNewConversation(): void {
    this.currentConversationId.set(null);
    this.messages.set([]);
    this.error.set(null);
  }

  sendMessage(): void {
    const message = this.messageInput().trim();
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
    this.messageInput.set('');

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
          }

          // Add AI response to messages
          this.messages.update(msgs => [...msgs, response.message]);
          this.isSending.set(false);

          // Reload conversations list
          this.loadConversations(pid);
        },
        error: (err) => {
          // Remove the optimistically added user message on error
          this.messages.update(msgs => msgs.slice(0, -1));
          this.messageInput.set(message); // Restore the message
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
}
