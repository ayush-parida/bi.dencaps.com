import { Component, inject, OnInit, OnDestroy, signal, effect } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Router, RouterLink, RouterLinkActive, RouterOutlet, NavigationEnd } from '@angular/router';
import { Subject, takeUntil, filter, forkJoin } from 'rxjs';
import { AuthService } from '../../core/services/auth.service';
import { ProjectService } from '../../core/services/project.service';
import { ChatService } from '../../core/services/chat.service';
import { PermissionService } from '../../core/services/permission.service';
import { Project, ConversationSummary } from '../../core/models';
import { HasPermissionDirective } from '../directives/permission.directive';
import { Permission } from '../../core/models/permission.model';

interface ConversationWithProject extends ConversationSummary {
  // project_id is already in ConversationSummary
}

@Component({
  selector: 'app-layout',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterLink, RouterLinkActive, RouterOutlet, HasPermissionDirective],
  templateUrl: './layout.html',
  styleUrl: './layout.scss',
})
export class Layout implements OnInit, OnDestroy {
  private readonly authService = inject(AuthService);
  private readonly projectService = inject(ProjectService);
  private readonly chatService = inject(ChatService);
  private readonly permissionService = inject(PermissionService);
  private readonly router = inject(Router);
  private readonly destroy$ = new Subject<void>();
  
  readonly Permission = Permission;
  
  currentUser$ = this.authService.currentUser$;
  
  projects = signal<Project[]>([]);
  allConversations = signal<ConversationWithProject[]>([]);
  selectedConversationId = signal<string | null>(null);
  loadingConversations = signal<boolean>(false);

  ngOnInit(): void {
    // Load global permissions first to enable navigation checks
    this.loadGlobalPermissions();
    this.loadProjects();
    
    // Listen for navigation changes to refresh conversations
    this.router.events.pipe(
      filter(event => event instanceof NavigationEnd),
      takeUntil(this.destroy$)
    ).subscribe(() => {
      this.loadAllConversations();
    });
  }

  loadGlobalPermissions(): void {
    this.permissionService.getMyPermissions()
      .pipe(takeUntil(this.destroy$))
      .subscribe({
        next: (permissions) => {
          console.log('Global permissions loaded:', permissions);
        },
        error: (err) => {
          console.error('Failed to load permissions:', err);
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
          // Load conversations for all projects
          this.loadAllConversations();
        },
        error: (err) => {
          console.error('Failed to load projects:', err);
        }
      });
  }

  loadAllConversations(): void {
    const projectList = this.projects();
    if (projectList.length === 0) {
      this.allConversations.set([]);
      return;
    }
    
    this.loadingConversations.set(true);
    
    // Load conversation summaries from all projects (lightweight, no messages)
    const requests = projectList.map(project => 
      this.chatService.getProjectConversationSummaries(project.project_id)
    );
    
    forkJoin(requests)
      .pipe(takeUntil(this.destroy$))
      .subscribe({
        next: (results) => {
          // Flatten conversations from all projects
          const allConvs: ConversationWithProject[] = [];
          results.forEach((conversations) => {
            conversations.forEach(conv => {
              allConvs.push(conv);
            });
          });
          
          // Sort by most recent
          allConvs.sort((a, b) => 
            new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
          );
          
          this.allConversations.set(allConvs);
          this.loadingConversations.set(false);
        },
        error: (err) => {
          console.error('Failed to load conversations:', err);
          this.allConversations.set([]);
          this.loadingConversations.set(false);
        }
      });
  }

  getProjectName(projectId: string): string {
    const project = this.projects().find(p => p.project_id === projectId);
    return project?.name || 'Unknown';
  }

  selectConversation(conv: ConversationWithProject): void {
    this.selectedConversationId.set(conv.conversation_id);
    this.router.navigate(['/chat'], { 
      queryParams: { 
        projectId: conv.project_id,
        conversationId: conv.conversation_id 
      } 
    });
  }

  deleteConversation(conv: ConversationWithProject, event: Event): void {
    event.stopPropagation(); // Prevent selecting the conversation
    
    if (!confirm('Are you sure you want to delete this conversation?')) {
      return;
    }

    this.chatService.deleteConversation(conv.conversation_id)
      .pipe(takeUntil(this.destroy$))
      .subscribe({
        next: () => {
          // Remove from local list
          this.allConversations.update(convs => 
            convs.filter(c => c.conversation_id !== conv.conversation_id)
          );
          
          // If this was the selected conversation, navigate away
          if (this.selectedConversationId() === conv.conversation_id) {
            this.selectedConversationId.set(null);
            this.router.navigate(['/chat']);
          }
        },
        error: (err) => {
          console.error('Failed to delete conversation:', err);
          alert('Failed to delete conversation: ' + err.message);
        }
      });
  }

  formatDate(dateInput: string | { $date: string } | any): string {
    if (!dateInput) return '';
    
    // Handle MongoDB BSON date format
    let dateString = dateInput;
    if (typeof dateInput === 'object' && dateInput.$date) {
      dateString = dateInput.$date;
    }
    
    const date = new Date(dateString);
    if (isNaN(date.getTime())) return '';
    
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    
    if (days === 0) {
      return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } else if (days === 1) {
      return 'Yesterday';
    } else if (days < 7) {
      return date.toLocaleDateString([], { weekday: 'short' });
    } else {
      return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
    }
  }

  logout(): void {
    this.authService.logout();
  }
}
