import { Component, Input, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DatasetData } from './rendering.models';

@Component({
  selector: 'app-dataset-renderer',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="dataset-container">
      <div class="dataset-header">
        <h4>{{ datasetData.name }}</h4>
        <p *ngIf="datasetData.description" class="description">{{ datasetData.description }}</p>
      </div>
      
      <div class="dataset-info">
        <span class="info-badge">{{ datasetData.columns.length }} columns</span>
        <span class="info-badge">{{ datasetData.rows.length }} rows</span>
      </div>

      <div class="table-wrapper">
        <table>
          <thead>
            <tr>
              <th *ngFor="let column of datasetData.columns">
                <div class="column-header">
                  <span class="column-name">{{ column.name }}</span>
                  <span class="column-type">{{ column.data_type }}</span>
                </div>
              </th>
            </tr>
          </thead>
          <tbody>
            <tr *ngFor="let row of datasetData.rows">
              <td *ngFor="let cell of row">{{ formatCell(cell) }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  `,
  styles: [`
    .dataset-container {
      margin: 1rem 0;
      border-radius: 8px;
      overflow: hidden;
      box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
      background: white;
    }

    .dataset-header {
      padding: 1rem 1.5rem;
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      color: white;
    }

    .dataset-header h4 {
      margin: 0 0 0.5rem 0;
      font-size: 1.2rem;
      font-weight: 600;
    }

    .description {
      margin: 0;
      font-size: 0.9rem;
      opacity: 0.9;
    }

    .dataset-info {
      padding: 0.75rem 1.5rem;
      background: #f5f5f5;
      border-bottom: 1px solid #ddd;
      display: flex;
      gap: 0.5rem;
    }

    .info-badge {
      padding: 0.25rem 0.75rem;
      background: white;
      border-radius: 12px;
      font-size: 0.85rem;
      font-weight: 500;
      color: #555;
      border: 1px solid #ddd;
    }

    .table-wrapper {
      overflow-x: auto;
      max-height: 400px;
      overflow-y: auto;
    }

    table {
      width: 100%;
      border-collapse: collapse;
      min-width: 600px;
    }

    thead {
      position: sticky;
      top: 0;
      background: #f9f9f9;
      z-index: 10;
    }

    .column-header {
      display: flex;
      flex-direction: column;
      gap: 0.25rem;
    }

    .column-name {
      font-weight: 600;
      color: #333;
    }

    .column-type {
      font-size: 0.75rem;
      color: #666;
      font-weight: normal;
      text-transform: uppercase;
    }

    th {
      padding: 0.75rem 1rem;
      text-align: left;
      border-bottom: 2px solid #ddd;
      white-space: nowrap;
    }

    tbody tr {
      border-bottom: 1px solid #eee;
      transition: background-color 0.2s;
    }

    tbody tr:hover {
      background: #f9f9f9;
    }

    tbody tr:last-child {
      border-bottom: none;
    }

    td {
      padding: 0.75rem 1rem;
      color: #555;
    }
  `]
})
export class DatasetRendererComponent implements OnInit {
  @Input() datasetData!: DatasetData;

  ngOnInit(): void {
    if (!this.datasetData) {
      throw new Error('datasetData is required');
    }
  }

  formatCell(cell: any): string {
    if (cell === null || cell === undefined) {
      return 'null';
    }
    if (typeof cell === 'object') {
      return JSON.stringify(cell);
    }
    return String(cell);
  }
}
