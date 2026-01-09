import {
  Directive,
  Input,
  TemplateRef,
  ViewContainerRef,
  OnInit,
  OnDestroy,
  inject,
  effect,
  ElementRef
} from '@angular/core';
import { Subscription } from 'rxjs';
import { PermissionService } from '../../core/services/permission.service';
import { Permission } from '../../core/models';

/**
 * Structural directive for conditionally rendering elements based on user permissions.
 * 
 * Basic usage - show if user has permission:
 * <button *hasPermission="Permission.ProjectCreate">Create Project</button>
 * 
 * Multiple permissions (ANY):
 * <div *hasPermission="[Permission.ProjectUpdate, Permission.ProjectDelete]">
 *   Edit/Delete Actions
 * </div>
 * 
 * Multiple permissions (ALL required):
 * <div *hasPermission="[Permission.AdminAccess, Permission.SystemSettings]; requireAll: true">
 *   System Settings
 * </div>
 * 
 * Project-scoped permission:
 * <button *hasPermission="Permission.ChatWrite; projectId: project.project_id">
 *   Send Message
 * </button>
 * 
 * With else template:
 * <button *hasPermission="Permission.ProjectDelete; else noAccess">Delete</button>
 * <ng-template #noAccess>
 *   <span class="disabled">No permission</span>
 * </ng-template>
 */
@Directive({
  selector: '[hasPermission]',
  standalone: true
})
export class HasPermissionDirective implements OnInit, OnDestroy {
  private readonly permissionService = inject(PermissionService);
  private readonly templateRef = inject(TemplateRef<any>);
  private readonly viewContainer = inject(ViewContainerRef);

  private permissions: Permission[] = [];
  private requireAll = false;
  private projectId?: string;
  private elseTemplateRef?: TemplateRef<any>;

  private subscription?: Subscription;
  private hasView = false;
  private showingElse = false;

  /**
   * The permission(s) to check. Can be a single Permission or array of Permissions.
   */
  @Input()
  set hasPermission(value: Permission | Permission[]) {
    this.permissions = Array.isArray(value) ? value : [value];
    this.updateView();
  }

  /**
   * If true, requires ALL permissions; if false, requires ANY permission.
   * Default: false (ANY)
   */
  @Input()
  set hasPermissionRequireAll(value: boolean) {
    this.requireAll = value;
    this.updateView();
  }

  /**
   * Optional project ID for project-scoped permission checks.
   */
  @Input()
  set hasPermissionProjectId(value: string | undefined) {
    this.projectId = value;
    this.updateView();
  }

  /**
   * Template to show when permission check fails.
   */
  @Input()
  set hasPermissionElse(templateRef: TemplateRef<any> | null) {
    this.elseTemplateRef = templateRef || undefined;
    this.updateView();
  }

  ngOnInit(): void {
    // Subscribe to permission changes
    this.subscription = this.permissionService.currentPermissions$.subscribe(() => {
      this.updateView();
    });
    
    // Initial check
    this.updateView();
  }

  ngOnDestroy(): void {
    this.subscription?.unsubscribe();
  }

  private updateView(): void {
    if (this.permissions.length === 0) {
      this.showMainTemplate();
      return;
    }

    const hasAccess = this.requireAll
      ? this.permissionService.hasAllPermissions(this.permissions, this.projectId)
      : this.permissionService.hasAnyPermission(this.permissions, this.projectId);

    if (hasAccess) {
      this.showMainTemplate();
    } else {
      this.showElseTemplate();
    }
  }

  private showMainTemplate(): void {
    if (!this.hasView) {
      this.viewContainer.clear();
      this.viewContainer.createEmbeddedView(this.templateRef);
      this.hasView = true;
      this.showingElse = false;
    }
  }

  private showElseTemplate(): void {
    if (this.hasView || (!this.showingElse && this.elseTemplateRef)) {
      this.viewContainer.clear();
      this.hasView = false;
      
      if (this.elseTemplateRef) {
        this.viewContainer.createEmbeddedView(this.elseTemplateRef);
        this.showingElse = true;
      }
    }
  }
}

/**
 * Attribute directive for disabling elements based on permissions.
 * Unlike *hasPermission, this keeps the element visible but disabled.
 * 
 * Usage:
 * <button [disableWithoutPermission]="Permission.ProjectDelete">Delete</button>
 * 
 * With project scope:
 * <button [disableWithoutPermission]="Permission.ChatWrite" [permissionProjectId]="projectId">
 *   Send
 * </button>
 */
@Directive({
  selector: '[disableWithoutPermission]',
  standalone: true
})
export class DisableWithoutPermissionDirective implements OnInit, OnDestroy {
  private readonly permissionService = inject(PermissionService);
  private readonly elementRef = inject(ElementRef);
  
  private permissions: Permission[] = [];
  private requireAll = false;
  private projectId?: string;
  private subscription?: Subscription;

  @Input()
  set disableWithoutPermission(value: Permission | Permission[]) {
    this.permissions = Array.isArray(value) ? value : [value];
    this.updateDisabledState();
  }

  @Input()
  set permissionRequireAll(value: boolean) {
    this.requireAll = value;
    this.updateDisabledState();
  }

  @Input()
  set permissionProjectId(value: string | undefined) {
    this.projectId = value;
    this.updateDisabledState();
  }

  ngOnInit(): void {
    this.subscription = this.permissionService.currentPermissions$.subscribe(() => {
      this.updateDisabledState();
    });
    this.updateDisabledState();
  }

  ngOnDestroy(): void {
    this.subscription?.unsubscribe();
  }

  private updateDisabledState(): void {
    if (this.permissions.length === 0) {
      return;
    }

    const hasAccess = this.requireAll
      ? this.permissionService.hasAllPermissions(this.permissions, this.projectId)
      : this.permissionService.hasAnyPermission(this.permissions, this.projectId);

    const el = this.elementRef.nativeElement;
    if (el) {
      if (hasAccess) {
        el.removeAttribute('disabled');
        el.classList.remove('permission-disabled');
      } else {
        el.setAttribute('disabled', 'true');
        el.classList.add('permission-disabled');
      }
    }
  }
}

/**
 * Pipe for checking permissions in templates.
 * 
 * Usage:
 * <ng-container *ngIf="Permission.ProjectCreate | hasPermission">
 *   <button>Create</button>
 * </ng-container>
 * 
 * Or inline:
 * [class.hidden]="!(Permission.ChatWrite | hasPermission:projectId)"
 */
import { Pipe, PipeTransform } from '@angular/core';

@Pipe({
  name: 'hasPermission',
  standalone: true,
  pure: false // Impure to react to permission changes
})
export class HasPermissionPipe implements PipeTransform {
  private readonly permissionService = inject(PermissionService);

  transform(permission: Permission | Permission[], projectId?: string, requireAll: boolean = false): boolean {
    const permissions = Array.isArray(permission) ? permission : [permission];
    
    if (requireAll) {
      return this.permissionService.hasAllPermissions(permissions, projectId);
    }
    return this.permissionService.hasAnyPermission(permissions, projectId);
  }
}

/**
 * Convenience type for exporting all permission utilities
 */
export const PERMISSION_DIRECTIVES = [
  HasPermissionDirective,
  DisableWithoutPermissionDirective,
  HasPermissionPipe
] as const;
