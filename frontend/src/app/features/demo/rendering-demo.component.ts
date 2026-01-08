import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ContentRendererComponent } from '../../shared/rendering';

@Component({
  selector: 'app-rendering-demo',
  standalone: true,
  imports: [CommonModule, ContentRendererComponent],
  template: `
    <div class="demo-container">
      <h1>Rendering Module Demo</h1>
      <p>This page demonstrates the various rendering capabilities of the module.</p>

      <section class="demo-section">
        <h2>1. Text Content</h2>
        <app-content-renderer [response]="textExample"></app-content-renderer>
      </section>

      <section class="demo-section">
        <h2>2. Bar Chart</h2>
        <app-content-renderer [response]="barChartExample"></app-content-renderer>
      </section>

      <section class="demo-section">
        <h2>3. Line Chart</h2>
        <app-content-renderer [response]="lineChartExample"></app-content-renderer>
      </section>

      <section class="demo-section">
        <h2>4. Pie Chart</h2>
        <app-content-renderer [response]="pieChartExample"></app-content-renderer>
      </section>

      <section class="demo-section">
        <h2>5. Mathematical Equation</h2>
        <app-content-renderer [response]="equationExample"></app-content-renderer>
      </section>

      <section class="demo-section">
        <h2>6. Table</h2>
        <app-content-renderer [response]="tableExample"></app-content-renderer>
      </section>

      <section class="demo-section">
        <h2>7. Dataset</h2>
        <app-content-renderer [response]="datasetExample"></app-content-renderer>
      </section>

      <section class="demo-section">
        <h2>8. Mixed Content</h2>
        <app-content-renderer [response]="mixedExample"></app-content-renderer>
      </section>
    </div>
  `,
  styles: [`
    .demo-container {
      max-width: 1200px;
      margin: 0 auto;
      padding: 2rem;
      background: #f5f5f5;
      min-height: 100vh;
    }

    h1 {
      color: #333;
      margin-bottom: 0.5rem;
    }

    h1 + p {
      color: #666;
      margin-bottom: 2rem;
    }

    .demo-section {
      margin-bottom: 3rem;
      padding: 1.5rem;
      background: white;
      border-radius: 8px;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    }

    .demo-section h2 {
      margin-top: 0;
      margin-bottom: 1rem;
      color: #444;
      border-bottom: 2px solid #e0e0e0;
      padding-bottom: 0.5rem;
    }
  `]
})
export class RenderingDemoComponent {
  textExample = {
    items: [
      {
        type: 'text',
        content: 'This is a text content example. It demonstrates how plain text responses are rendered with basic formatting support.'
      }
    ]
  };

  barChartExample = {
    items: [
      {
        type: 'chart',
        data: {
          chart_type: 'bar',
          title: 'Monthly Sales Performance',
          labels: ['January', 'February', 'March', 'April', 'May', 'June'],
          datasets: [
            {
              label: '2024 Sales ($)',
              data: [12000, 15000, 13000, 17000, 19000, 21000],
              background_color: 'rgba(54, 162, 235, 0.6)',
              border_color: 'rgba(54, 162, 235, 1)'
            }
          ]
        }
      }
    ]
  };

  lineChartExample = {
    items: [
      {
        type: 'chart',
        data: {
          chart_type: 'line',
          title: 'Website Traffic Trend',
          labels: ['Week 1', 'Week 2', 'Week 3', 'Week 4', 'Week 5'],
          datasets: [
            {
              label: 'Visitors',
              data: [1200, 1500, 1800, 2200, 2500],
              border_color: 'rgba(75, 192, 192, 1)',
              background_color: 'rgba(75, 192, 192, 0.2)'
            }
          ]
        }
      }
    ]
  };

  pieChartExample = {
    items: [
      {
        type: 'chart',
        data: {
          chart_type: 'pie',
          title: 'Market Share Distribution',
          labels: ['Product A', 'Product B', 'Product C', 'Product D'],
          datasets: [
            {
              label: 'Market Share (%)',
              data: [30, 25, 20, 25]
            }
          ]
        }
      }
    ]
  };

  equationExample = {
    items: [
      {
        type: 'text',
        content: 'The famous Einstein equation:'
      },
      {
        type: 'equation',
        latex: 'E = mc^2',
        display: true
      },
      {
        type: 'text',
        content: 'And the quadratic formula:'
      },
      {
        type: 'equation',
        latex: 'x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}',
        display: true
      }
    ]
  };

  tableExample = {
    items: [
      {
        type: 'table',
        data: {
          headers: ['Product', 'Revenue', 'Growth', 'Status'],
          rows: [
            ['Widget A', '$45,000', '+12%', 'Active'],
            ['Widget B', '$32,000', '+8%', 'Active'],
            ['Widget C', '$28,000', '-3%', 'Review'],
            ['Widget D', '$51,000', '+15%', 'Active'],
            ['Widget E', '$39,000', '+5%', 'Active']
          ]
        }
      }
    ]
  };

  datasetExample = {
    items: [
      {
        type: 'dataset',
        data: {
          name: 'Top Customers by Revenue',
          description: 'Q1 2024 customer analytics data',
          columns: [
            { name: 'customer_id', data_type: 'integer' },
            { name: 'company_name', data_type: 'string' },
            { name: 'revenue', data_type: 'float' },
            { name: 'region', data_type: 'string' },
            { name: 'contract_type', data_type: 'string' }
          ],
          rows: [
            [1001, 'Acme Corporation', 125000.50, 'North America', 'Enterprise'],
            [1002, 'TechStart Inc', 98000.25, 'Europe', 'Professional'],
            [1003, 'Global Industries', 156000.00, 'Asia Pacific', 'Enterprise'],
            [1004, 'Metro Solutions', 87500.75, 'South America', 'Professional'],
            [1005, 'Digital Dynamics', 142000.00, 'North America', 'Enterprise']
          ]
        }
      }
    ]
  };

  mixedExample = {
    items: [
      {
        type: 'text',
        content: 'Here is a comprehensive sales analysis for Q1 2024:'
      },
      {
        type: 'chart',
        data: {
          chart_type: 'bar',
          title: 'Q1 2024 Monthly Sales',
          labels: ['January', 'February', 'March'],
          datasets: [
            {
              label: 'Sales ($)',
              data: [45000, 52000, 48000]
            }
          ]
        }
      },
      {
        type: 'text',
        content: 'The average monthly sales can be calculated as:'
      },
      {
        type: 'equation',
        latex: '\\text{Average} = \\frac{\\sum_{i=1}^{n} x_i}{n} = \\frac{145000}{3} = 48333',
        display: true
      },
      {
        type: 'text',
        content: 'Key metrics summary:'
      },
      {
        type: 'table',
        data: {
          headers: ['Metric', 'Value', 'Change'],
          rows: [
            ['Total Sales', '$145,000', '+8.5%'],
            ['Average Sales', '$48,333', '+6.2%'],
            ['Best Month', 'February', '$52,000'],
            ['Growth Rate', '8.5%', '+2.1pp']
          ]
        }
      }
    ]
  };
}
