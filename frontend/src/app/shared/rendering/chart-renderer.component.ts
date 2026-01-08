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
      <h4 *ngIf="chartData.title">{{ chartData.title }}</h4>
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

    h4 {
      margin: 0 0 1rem 0;
      color: #333;
      font-size: 1.1rem;
      font-weight: 600;
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

    const datasets = this.chartData.datasets.map(ds => ({
      label: ds.label,
      data: ds.data,
      backgroundColor: ds.background_color || this.generateColor(0.6),
      borderColor: ds.border_color || this.generateColor(1),
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

  private generateColor(opacity: number): string | string[] {
    if (this.chartData.chart_type === 'pie') {
      // Generate multiple colors for pie chart
      return this.chartData.labels.map((_, i) => {
        const hue = (i * 360) / this.chartData.labels.length;
        return `hsla(${hue}, 70%, 60%, ${opacity})`;
      });
    } else {
      // Single color for bar/line chart
      return `rgba(54, 162, 235, ${opacity})`;
    }
  }
}
