import { Component, Input, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TableData } from './rendering.models';

@Component({
  selector: 'app-table-renderer',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="table-container">
      <div class="table-wrapper">
        <table>
          <thead>
            <tr>
              <th *ngFor="let header of tableData.headers">{{ header }}</th>
            </tr>
          </thead>
          <tbody>
            <tr *ngFor="let row of tableData.rows">
              <td *ngFor="let cell of row">{{ cell }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  `,
  styles: [`
    .table-container {
      margin: 1rem 0;
      border-radius: 8px;
      overflow: hidden;
      box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    }

    .table-wrapper {
      overflow-x: auto;
    }

    table {
      width: 100%;
      border-collapse: collapse;
      background: white;
      min-width: 400px;
    }

    thead {
      background: #f5f5f5;
    }

    th {
      padding: 0.75rem 1rem;
      text-align: left;
      font-weight: 600;
      color: #333;
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
export class TableRendererComponent implements OnInit {
  @Input() tableData!: TableData;

  ngOnInit(): void {
    if (!this.tableData) {
      throw new Error('tableData is required');
    }
  }
}
