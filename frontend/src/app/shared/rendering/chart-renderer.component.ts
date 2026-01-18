import { Component, Input, OnInit, OnDestroy, ElementRef, ViewChild, AfterViewInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Chart, ChartConfiguration, registerables } from 'chart.js';
import { ChartData } from './rendering.models';

Chart.register(...registerables);

@Component({
  selector: 'app-chart-renderer',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="chart-container">
      <div class="chart-header">
        <h4 *ngIf="chartData.title">{{ chartData.title }}</h4>
        <button class="download-chart-btn" (click)="downloadChart()" title="Download chart as image">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
            <polyline points="7 10 12 15 17 10"></polyline>
            <line x1="12" y1="15" x2="12" y2="3"></line>
          </svg>
        </button>
      </div>
      <canvas #chartCanvas></canvas>
    </div>
  `,
  styles: [`
    .chart-container {
      margin: 1rem 0;
      padding: 1rem;
      background: white;
      border-radius: 8px;
      box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    }

    .chart-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 1rem;
    }

    h4 {
      margin: 0;
      color: #333;
      font-size: 1.1rem;
      font-weight: 600;
    }

    .download-chart-btn {
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 0.5rem;
      background: transparent;
      color: #666;
      border: 1px solid #ddd;
      border-radius: 6px;
      cursor: pointer;
      transition: all 0.2s ease;
    }

    .download-chart-btn:hover {
      background: #f5f5f5;
      color: #0066cc;
      border-color: #0066cc;
      transform: translateY(-1px);
      box-shadow: 0 2px 6px rgba(0, 0, 0, 0.1);
    }

    .download-chart-btn:active {
      transform: translateY(0);
    }

    .download-chart-btn svg {
      width: 18px;
      height: 18px;
    }

    canvas {
      max-width: 100%;
      height: auto !important;
    }
  `]
})
export class ChartRendererComponent implements OnInit, AfterViewInit, OnDestroy {
  @Input() chartData!: ChartData;
  @ViewChild('chartCanvas', { static: false }) chartCanvas!: ElementRef<HTMLCanvasElement>;
  
  private chart?: Chart;

  ngOnInit(): void {
    if (!this.chartData) {
      throw new Error('chartData is required');
    }
  }

  ngAfterViewInit(): void {
    this.renderChart();
  }

  ngOnDestroy(): void {
    if (this.chart) {
      this.chart.destroy();
    }
  }

  private renderChart(): void {
    if (!this.chartCanvas) {
      return;
    }

    const ctx = this.chartCanvas.nativeElement.getContext('2d');
    if (!ctx) {
      return;
    }

    const datasets = this.chartData.datasets.map((ds, index) => ({
      label: ds.label,
      data: ds.data,
      backgroundColor: ds.background_color || this.generateColor(index, 0.6),
      borderColor: ds.border_color || this.generateColor(index, 1),
      borderWidth: 2
    }));

    const config: ChartConfiguration = {
      type: this.chartData.chart_type === 'bar' ? 'bar' : 
            this.chartData.chart_type === 'line' ? 'line' : 'pie',
      data: {
        labels: this.chartData.labels,
        datasets: datasets
      },
      options: {
        responsive: true,
        maintainAspectRatio: true,
        plugins: {
          legend: {
            display: true,
            position: 'top'
          },
          tooltip: {
            enabled: true
          }
        },
        ...(this.chartData.chart_type !== 'pie' && {
          scales: {
            y: {
              beginAtZero: true
            }
          }
        })
      }
    };

    this.chart = new Chart(ctx, config);
  }

  private generateColor(index: number, opacity: number): string | string[] {
    const colorPalette = [
      { h: 234, s: 82, l: 65 },  // Blue
      { h: 321, s: 88, l: 79 },  // Pink
      { h: 201, s: 98, l: 65 },  // Cyan
      { h: 147, s: 79, l: 57 },  // Green
      { h: 334, s: 94, l: 70 },  // Rose
      { h: 50, s: 98, l: 63 },   // Yellow
      { h: 190, s: 77, l: 50 },  // Teal
      { h: 168, s: 76, l: 80 },  // Light Blue
      { h: 5, s: 85, l: 80 },    // Light Red
      { h: 41, s: 100, l: 88 },  // Light Orange
      { h: 287, s: 50, l: 63 },  // Purple
      { h: 330, s: 88, l: 84 },  // Light Pink
      { h: 147, s: 72, l: 69 },  // Light Green
      { h: 201, s: 77, l: 78 },  // Sky Blue
      { h: 321, s: 44, l: 68 }   // Mauve
    ];

    if (this.chartData.chart_type === 'pie') {
      // Generate multiple colors for pie chart slices
      return this.chartData.labels.map((_, i) => {
        const color = colorPalette[i % colorPalette.length];
        return `hsla(${color.h}, ${color.s}%, ${color.l}%, ${opacity})`;
      });
    } else {
      // Generate a single color for each dataset in bar/line charts
      const color = colorPalette[index % colorPalette.length];
      return `hsla(${color.h}, ${color.s}%, ${color.l}%, ${opacity})`;
    }
  }

  downloadChart(): void {
    if (!this.chart || !this.chartCanvas) {
      return;
    }

    // Get the canvas element
    const canvas = this.chartCanvas.nativeElement;
    
    // Convert canvas to blob
    canvas.toBlob((blob) => {
      if (!blob) {
        console.error('Failed to create image blob');
        return;
      }

      // Create download link
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      const filename = this.chartData.title 
        ? `${this.chartData.title.replace(/[^a-z0-9]/gi, '_').toLowerCase()}.png`
        : `chart_${Date.now()}.png`;
      
      link.download = filename;
      link.href = url;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      
      // Clean up
      URL.revokeObjectURL(url);
    }, 'image/png');
  }
}
