# Rendering Module - Examples & Usage

## Overview
The rendering module provides a structured way to display AI responses with rich content including charts, equations, tables, and datasets.

## Response Format

All responses must follow this JSON schema:

```json
{
  "items": [
    {
      "type": "text|chart|equation|table|dataset",
      ...type-specific fields
    }
  ]
}
```

## Examples

### 1. Text Response

```json
{
  "items": [
    {
      "type": "text",
      "content": "This is a simple text response with explanations."
    }
  ]
}
```

### 2. Chart Response

#### Bar Chart
```json
{
  "items": [
    {
      "type": "chart",
      "data": {
        "chart_type": "bar",
        "title": "Monthly Sales Performance",
        "labels": ["Jan", "Feb", "Mar", "Apr", "May"],
        "datasets": [
          {
            "label": "2024 Sales",
            "data": [12000, 15000, 13000, 17000, 19000],
            "background_color": "rgba(54, 162, 235, 0.6)",
            "border_color": "rgba(54, 162, 235, 1)"
          }
        ]
      }
    }
  ]
}
```

#### Line Chart
```json
{
  "items": [
    {
      "type": "chart",
      "data": {
        "chart_type": "line",
        "title": "User Growth Trend",
        "labels": ["Week 1", "Week 2", "Week 3", "Week 4"],
        "datasets": [
          {
            "label": "New Users",
            "data": [120, 150, 180, 220],
            "border_color": "rgba(75, 192, 192, 1)"
          }
        ]
      }
    }
  ]
}
```

#### Pie Chart
```json
{
  "items": [
    {
      "type": "chart",
      "data": {
        "chart_type": "pie",
        "title": "Market Share Distribution",
        "labels": ["Product A", "Product B", "Product C", "Product D"],
        "datasets": [
          {
            "label": "Market Share",
            "data": [30, 25, 20, 25]
          }
        ]
      }
    }
  ]
}
```

### 3. Equation Response

#### Inline Equation
```json
{
  "items": [
    {
      "type": "equation",
      "latex": "E = mc^2",
      "display": false
    }
  ]
}
```

#### Display Equation
```json
{
  "items": [
    {
      "type": "equation",
      "latex": "\\int_{-\\infty}^{\\infty} e^{-x^2} dx = \\sqrt{\\pi}",
      "display": true
    }
  ]
}
```

### 4. Table Response

```json
{
  "items": [
    {
      "type": "table",
      "data": {
        "headers": ["Product", "Revenue", "Growth", "Status"],
        "rows": [
          ["Widget A", "$45,000", "+12%", "Active"],
          ["Widget B", "$32,000", "+8%", "Active"],
          ["Widget C", "$28,000", "-3%", "Review"],
          ["Widget D", "$51,000", "+15%", "Active"]
        ]
      }
    }
  ]
}
```

### 5. Dataset Response

```json
{
  "items": [
    {
      "type": "dataset",
      "data": {
        "name": "Customer Analytics Dataset",
        "description": "Top customers by revenue",
        "columns": [
          {"name": "customer_id", "data_type": "integer"},
          {"name": "name", "data_type": "string"},
          {"name": "revenue", "data_type": "float"},
          {"name": "region", "data_type": "string"}
        ],
        "rows": [
          [1001, "Acme Corp", 125000.50, "North"],
          [1002, "TechStart Inc", 98000.25, "East"],
          [1003, "Global Industries", 156000.00, "West"],
          [1004, "Metro Solutions", 87500.75, "South"]
        ]
      }
    }
  ]
}
```

### 6. Mixed Response (Combining Multiple Types)

```json
{
  "items": [
    {
      "type": "text",
      "content": "Here's an analysis of your sales data for Q1 2024:"
    },
    {
      "type": "chart",
      "data": {
        "chart_type": "bar",
        "title": "Q1 2024 Sales by Month",
        "labels": ["January", "February", "March"],
        "datasets": [
          {
            "label": "Sales ($)",
            "data": [45000, 52000, 48000]
          }
        ]
      }
    },
    {
      "type": "text",
      "content": "The average monthly sales can be calculated using:"
    },
    {
      "type": "equation",
      "latex": "\\text{Average} = \\frac{\\sum_{i=1}^{n} x_i}{n}",
      "display": true
    },
    {
      "type": "table",
      "data": {
        "headers": ["Metric", "Value"],
        "rows": [
          ["Total Sales", "$145,000"],
          ["Average", "$48,333"],
          ["Growth Rate", "+8.5%"]
        ]
      }
    }
  ]
}
```

## Backend Usage (Rust)

### AI Service Integration

The AI service now includes a `process_chat_message_structured` method that enforces structured responses:

```rust
use crate::services::AIService;
use crate::models::StructuredResponse;

let ai_service = AIService::new(api_url, model_name);

match ai_service.process_chat_message_structured(
    "Show me sales trends",
    None
).await {
    Ok(structured_response) => {
        // Response is validated and ready to send to frontend
        println!("Valid structured response with {} items", structured_response.items.len());
    }
    Err(e) => {
        // Malformed response rejected
        eprintln!("Invalid AI response: {}", e);
    }
}
```

### Validation

The backend automatically validates:
- Response must contain at least one item
- Each item must have a valid type
- Chart data lengths must match (labels vs datasets)
- Table/dataset rows must match header/column counts
- All required fields must be present

## Frontend Usage (Angular)

### Automatic Rendering

The content renderer automatically detects and renders structured responses:

```typescript
import { ContentRendererComponent } from './shared/rendering';

// In your component
<app-content-renderer [response]="structuredResponse"></app-content-renderer>
```

### Fallback for Plain Text

If the response is not structured JSON, it falls back to plain text display:

```typescript
// The renderer will try to parse JSON, and if it fails, display as text
tryParseStructuredContent(content: string): any {
  try {
    return JSON.parse(content);
  } catch {
    return null;
  }
}
```

## Error Handling

The rendering module includes comprehensive error handling:

1. **Backend Validation**: Rejects malformed responses before sending to frontend
2. **Frontend Parsing**: Validates structure before rendering
3. **Component-Level**: Each renderer validates its specific data requirements
4. **User Feedback**: Clear error messages displayed when rendering fails

## Best Practices

1. **Always validate**: Use the backend validation before sending responses
2. **Meaningful labels**: Use clear, descriptive labels for charts and tables
3. **Appropriate types**: Choose the right visualization for your data
4. **Mixed content**: Combine text with visuals for better explanations
5. **Error messages**: Provide helpful error messages when validation fails

## Security Considerations

1. **No script execution**: HTML content is sanitized to prevent XSS
2. **Validated schemas**: All data structures are validated before rendering
3. **Safe LaTeX rendering**: KaTeX is configured to prevent code execution
4. **Type safety**: TypeScript ensures type correctness throughout

## Performance

- **Lazy loading**: Rendering components are loaded on-demand
- **Efficient updates**: Angular's change detection optimized
- **Chart destruction**: Charts are properly destroyed to prevent memory leaks
- **Responsive**: All renderers are mobile-friendly
