import { Component, Input, OnInit, ElementRef, ViewChild, AfterViewInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import * as katex from 'katex';

@Component({
  selector: 'app-equation-renderer',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="equation-container" [class.display-mode]="display">
      <div #equationElement></div>
    </div>
  `,
  styles: [`
    .equation-container {
      margin: 1rem 0;
      padding: 1rem;
      background: #f9f9f9;
      border-left: 4px solid #4CAF50;
      border-radius: 4px;
      overflow-x: auto;
    }

    .equation-container.display-mode {
      text-align: center;
      padding: 1.5rem;
    }

    :host ::ng-deep .katex {
      font-size: 1.1em;
    }

    :host ::ng-deep .katex-display {
      margin: 0;
    }
  `]
})
export class EquationRendererComponent implements OnInit, AfterViewInit {
  @Input() latex!: string;
  @Input() display: boolean = false;
  @ViewChild('equationElement', { static: false }) equationElement!: ElementRef<HTMLDivElement>;

  ngOnInit(): void {
    if (!this.latex) {
      throw new Error('latex is required');
    }
  }

  ngAfterViewInit(): void {
    this.renderEquation();
  }

  private renderEquation(): void {
    if (!this.equationElement) {
      return;
    }

    try {
      katex.render(this.latex, this.equationElement.nativeElement, {
        displayMode: this.display,
        throwOnError: false,
        errorColor: '#cc0000',
        strict: false,
        trust: false
      });
    } catch (error) {
      console.error('Error rendering equation:', error);
      this.equationElement.nativeElement.textContent = `Error rendering equation: ${this.latex}`;
      this.equationElement.nativeElement.style.color = 'red';
    }
  }
}
