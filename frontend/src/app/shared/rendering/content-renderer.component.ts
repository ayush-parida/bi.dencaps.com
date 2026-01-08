import { Component, Input, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DomSanitizer, SafeHtml } from '@angular/platform-browser';
import { RenderItem, StructuredResponse } from './rendering.models';
import { RenderingService } from './rendering.service';
import { ChartRendererComponent } from './chart-renderer.component';
import { EquationRendererComponent } from './equation-renderer.component';
import { TableRendererComponent } from './table-renderer.component';
import { DatasetRendererComponent } from './dataset-renderer.component';

@Component({
  selector: 'app-content-renderer',
  standalone: true,
  imports: [
    CommonModule,
    ChartRendererComponent,
    EquationRendererComponent,
    TableRendererComponent,
    DatasetRendererComponent
  ],
  template: `
    <div class="content-renderer">
      <div *ngIf="error" class="error-message">
        <strong>Error:</strong> {{ error }}
      </div>

      <div *ngIf="!error && structuredResponse" class="render-items">
        <ng-container *ngFor="let item of structuredResponse.items; trackBy: trackByIndex">
          
          <!-- Text Content -->
          <div *ngIf="item.type === 'text'" class="text-content" [innerHTML]="sanitizeHtml(item.content)"></div>

          <!-- Chart Content -->
          <app-chart-renderer *ngIf="item.type === 'chart'" [chartData]="item.data"></app-chart-renderer>

          <!-- Equation Content -->
          <app-equation-renderer 
            *ngIf="item.type === 'equation'" 
            [latex]="item.latex" 
            [display]="item.display !== false">
          </app-equation-renderer>

          <!-- Table Content -->
          <app-table-renderer *ngIf="item.type === 'table'" [tableData]="item.data"></app-table-renderer>

          <!-- Dataset Content -->
          <app-dataset-renderer *ngIf="item.type === 'dataset'" [datasetData]="item.data"></app-dataset-renderer>

        </ng-container>
      </div>
    </div>
  `,
  styles: [`
    .content-renderer {
      width: 100%;
    }

    .error-message {
      padding: 1rem;
      background: #ffebee;
      border-left: 4px solid #f44336;
      border-radius: 4px;
      color: #c62828;
      margin-bottom: 1rem;
    }

    .error-message strong {
      font-weight: 600;
    }

    .render-items {
      display: flex;
      flex-direction: column;
      gap: 1rem;
    }

    .text-content {
      padding: 1rem;
      line-height: 1.6;
      color: #333;
      background: white;
      border-radius: 4px;
    }

    :host ::ng-deep .text-content p {
      margin: 0 0 0.5rem 0;
    }

    :host ::ng-deep .text-content p:last-child {
      margin-bottom: 0;
    }

    :host ::ng-deep .text-content strong {
      font-weight: 600;
      color: #000;
    }

    :host ::ng-deep .text-content code {
      background: #f5f5f5;
      padding: 0.2rem 0.4rem;
      border-radius: 3px;
      font-family: 'Courier New', monospace;
      font-size: 0.9em;
    }

    :host ::ng-deep .text-content pre {
      background: #f5f5f5;
      padding: 1rem;
      border-radius: 4px;
      overflow-x: auto;
    }

    :host ::ng-deep .text-content ul,
    :host ::ng-deep .text-content ol {
      margin: 0.5rem 0;
      padding-left: 1.5rem;
    }

    :host ::ng-deep .text-content li {
      margin: 0.25rem 0;
    }
  `]
})
export class ContentRendererComponent implements OnInit {
  @Input() response: any;
  
  structuredResponse?: StructuredResponse;
  error?: string;

  constructor(
    private renderingService: RenderingService,
    private sanitizer: DomSanitizer
  ) {}

  ngOnInit(): void {
    this.parseResponse();
  }

  private parseResponse(): void {
    try {
      this.structuredResponse = this.renderingService.parseStructuredResponse(this.response);
      this.error = undefined;
    } catch (e: any) {
      console.error('Failed to parse structured response:', e);
      this.error = e.message || 'Failed to parse response';
      this.structuredResponse = undefined;
    }
  }

  sanitizeHtml(content: string): SafeHtml {
    // Sanitize HTML content to prevent XSS attacks
    // This removes dangerous elements and attributes while preserving safe formatting
    return this.sanitizer.sanitize(1, content) || ''; // 1 is SecurityContext.HTML
  }

  trackByIndex(index: number): number {
    return index;
  }
}
