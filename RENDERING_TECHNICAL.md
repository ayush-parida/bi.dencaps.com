# Rendering Module - Technical Documentation

## Overview

The Rendering Module provides a production-ready system for interpreting and rendering structured AI responses in the DencapsBI platform. It supports multiple content types including charts, mathematical equations, tables, and datasets with strict schema validation.

## Architecture

### Backend (Rust)

The backend enforces response schemas and validates all AI responses before sending them to the frontend.

#### Models (`backend/src/models/mod.rs`)

```rust
// Main rendering content enum
pub enum RenderContent {
    Text { content: String },
    Chart { data: ChartData },
    Equation { latex: String, display: Option<bool> },
    Table { data: TableData },
    Dataset { data: DatasetData },
}

// Structured response wrapper
pub struct StructuredResponse {
    pub items: Vec<RenderContent>,
}
```

#### Validation

The `StructuredResponse::validate_content()` method enforces:
- At least one item must be present
- Text content cannot be empty
- Chart labels and datasets must match in length
- Table rows must match header count
- Dataset rows must match column count
- All required fields must be present

Example validation:
```rust
let response = StructuredResponse { items: vec![...] };
match response.validate_content() {
    Ok(()) => { /* Valid response */ }
    Err(e) => { /* Invalid: {e} */ }
}
```

#### AI Service Integration (`backend/src/services/ai.rs`)

Two methods are provided:

1. **`process_chat_message`** - Returns plain text (backward compatible)
2. **`process_chat_message_structured`** - Returns validated structured response

Example usage:
```rust
let ai_service = AIService::new(api_url, model_name);

// Get structured response
match ai_service.process_chat_message_structured(
    "Show me sales trends with a chart",
    None
).await {
    Ok(structured) => {
        // Response is validated and ready to send
        serde_json::to_string(&structured)?
    }
    Err(e) => {
        // Malformed response rejected
        log::error!("Invalid AI response: {}", e);
    }
}
```

The system message instructs the AI to respond with proper JSON format and explains each content type.

### Frontend (Angular)

The frontend provides a modular, type-safe rendering system with automatic content detection.

#### Models (`frontend/src/app/shared/rendering/rendering.models.ts`)

TypeScript interfaces mirror the Rust backend models:

```typescript
export interface StructuredResponse {
  items: RenderItem[];
}

export type RenderItem = 
  | TextContent 
  | ChartContent 
  | EquationContent 
  | TableContent 
  | DatasetContent;
```

#### Rendering Service (`frontend/src/app/shared/rendering/rendering.service.ts`)

Provides validation before rendering:

```typescript
parseStructuredResponse(response: any): StructuredResponse {
  // Validates structure
  // Throws descriptive errors on invalid data
  return validatedResponse;
}
```

#### Content Renderer Component (`frontend/src/app/shared/rendering/content-renderer.component.ts`)

Main component that dynamically renders based on content type:

```typescript
@Component({
  selector: 'app-content-renderer',
  template: `
    <div *ngFor="let item of structuredResponse.items">
      <app-chart-renderer *ngIf="item.type === 'chart'" [chartData]="item.data">
      <app-equation-renderer *ngIf="item.type === 'equation'" [latex]="item.latex">
      <app-table-renderer *ngIf="item.type === 'table'" [tableData]="item.data">
      <app-dataset-renderer *ngIf="item.type === 'dataset'" [datasetData]="item.data">
      <div *ngIf="item.type === 'text'" [innerHTML]="sanitizeHtml(item.content)">
    </div>
  `
})
```

## Content Types

### 1. Text

Simple text content with basic HTML formatting support.

**Backend Model:**
```rust
RenderContent::Text { 
    content: String 
}
```

**Frontend Interface:**
```typescript
interface TextContent {
  type: 'text';
  content: string;
}
```

**Example:**
```json
{
  "type": "text",
  "content": "This is a text response"
}
```

### 2. Chart

Supports bar, line, and pie charts using Chart.js.

**Backend Model:**
```rust
RenderContent::Chart { 
    data: ChartData {
        chart_type: ChartType, // Bar, Line, or Pie
        title: Option<String>,
        labels: Vec<String>,
        datasets: Vec<ChartDataset>,
    }
}
```

**Frontend Component:** `ChartRendererComponent`
- Uses Chart.js for rendering
- Automatically generates colors for pie charts
- Responsive and accessible
- Properly destroys charts on component destruction

**Example:**
```json
{
  "type": "chart",
  "data": {
    "chart_type": "bar",
    "title": "Monthly Sales",
    "labels": ["Jan", "Feb", "Mar"],
    "datasets": [{
      "label": "Sales",
      "data": [1000, 1500, 1200]
    }]
  }
}
```

### 3. Equation

Mathematical equations rendered with KaTeX (LaTeX syntax).

**Backend Model:**
```rust
RenderContent::Equation { 
    latex: String,
    display: Option<bool> // true for display mode, false for inline
}
```

**Frontend Component:** `EquationRendererComponent`
- Uses KaTeX for rendering
- Supports both inline and display modes
- Safe rendering (no code execution)
- Error handling with fallback display

**Example:**
```json
{
  "type": "equation",
  "latex": "E = mc^2",
  "display": true
}
```

### 4. Table

Tabular data with headers and rows.

**Backend Model:**
```rust
RenderContent::Table { 
    data: TableData {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    }
}
```

**Frontend Component:** `TableRendererComponent`
- Responsive table with horizontal scrolling
- Hover effects for better UX
- Styled headers and rows
- Mobile-friendly

**Example:**
```json
{
  "type": "table",
  "data": {
    "headers": ["Name", "Value"],
    "rows": [
      ["Item 1", "100"],
      ["Item 2", "200"]
    ]
  }
}
```

### 5. Dataset

Structured datasets with typed columns and metadata.

**Backend Model:**
```rust
RenderContent::Dataset { 
    data: DatasetData {
        name: String,
        description: Option<String>,
        columns: Vec<ColumnInfo>,
        rows: Vec<Vec<serde_json::Value>>,
    }
}
```

**Frontend Component:** `DatasetRendererComponent`
- Shows column names with data types
- Displays row and column counts
- Scrollable for large datasets
- Rich visual design with gradient header

**Example:**
```json
{
  "type": "dataset",
  "data": {
    "name": "Customer Data",
    "description": "Top customers",
    "columns": [
      {"name": "id", "data_type": "integer"},
      {"name": "name", "data_type": "string"}
    ],
    "rows": [
      [1, "Alice"],
      [2, "Bob"]
    ]
  }
}
```

## Integration

### Chat Interface Integration

The chat interface automatically detects and renders structured responses:

```typescript
// In chat-interface.component.ts
@if (tryParseStructuredContent(message.content); as structuredData) {
  <app-content-renderer [response]="structuredData"></app-content-renderer>
} @else {
  <div class="message-text">{{ message.content }}</div>
}
```

### Analytics Query Interface Integration

Analytics responses are also automatically rendered:

```typescript
// In query-interface.component.ts
@if (tryParseStructuredResponse(currentResponse); as structuredData) {
  <app-content-renderer [response]="structuredData"></app-content-renderer>
} @else {
  <pre>{{ currentResponse }}</pre>
}
```

## Error Handling

### Backend Errors

1. **Validation Errors**: Malformed responses are rejected with descriptive error messages
2. **Parsing Errors**: JSON parsing failures are caught and reported
3. **Type Errors**: Invalid content types are detected and rejected

Example error messages:
- "Response must contain at least one item"
- "Chart must have at least one label"
- "Dataset length must match labels length"
- "All table rows must match header length"

### Frontend Errors

1. **Parsing Errors**: Invalid JSON structure displays error message
2. **Validation Errors**: Schema violations show descriptive errors
3. **Rendering Errors**: Component-level errors with fallback display

Error display:
```html
<div class="error-message">
  <strong>Error:</strong> {{ error }}
</div>
```

## Security

### Backend Security

1. **Schema Validation**: All responses validated before sending
2. **Type Safety**: Rust's type system prevents invalid data structures
3. **No Code Execution**: Only data validation, no dynamic code execution

### Frontend Security

1. **HTML Sanitization**: Script tags removed from text content
2. **Safe LaTeX Rendering**: KaTeX configured to prevent code execution
3. **Input Validation**: All inputs validated before rendering
4. **Type Safety**: TypeScript ensures type correctness

## Performance

### Optimization Strategies

1. **Lazy Loading**: Rendering components loaded on-demand
2. **Chart Destruction**: Charts properly destroyed to prevent memory leaks
3. **Efficient Updates**: Angular change detection optimized
4. **Responsive Design**: All renderers mobile-friendly

### Bundle Sizes

- Initial bundle: ~299 KB (gzipped: ~79 KB)
- Rendering module: Included in lazy-loaded chunks
- Chart.js: ~491 KB (lazy-loaded when needed)
- KaTeX CSS: ~24 KB

## Testing

### Backend Testing

Test validation logic:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_chart_response() {
        let response = StructuredResponse {
            items: vec![RenderContent::Chart { /* ... */ }]
        };
        assert!(response.validate_content().is_ok());
    }

    #[test]
    fn test_invalid_chart_mismatched_lengths() {
        let response = StructuredResponse {
            items: vec![RenderContent::Chart { 
                data: ChartData {
                    labels: vec!["A".to_string()],
                    datasets: vec![ChartDataset {
                        data: vec![1.0, 2.0], // Wrong length
                        /* ... */
                    }]
                }
            }]
        };
        assert!(response.validate_content().is_err());
    }
}
```

### Frontend Testing

Use the demo component at `/demo/rendering` to visually test all content types:

```bash
cd frontend
npm start
# Navigate to http://localhost:4200/demo/rendering
```

## API Reference

### Backend

**`AIService::process_chat_message_structured`**
- **Input**: `message: &str, context: Option<&str>`
- **Output**: `Result<StructuredResponse, String>`
- **Purpose**: Process chat message and return validated structured response

**`StructuredResponse::validate_content`**
- **Input**: `&self`
- **Output**: `Result<(), String>`
- **Purpose**: Validate the structured response content

### Frontend

**`RenderingService::parseStructuredResponse`**
- **Input**: `response: any`
- **Output**: `StructuredResponse`
- **Purpose**: Parse and validate response before rendering
- **Throws**: Error with descriptive message on invalid data

**`ContentRendererComponent`**
- **Input**: `[response]="structuredResponse"`
- **Purpose**: Main rendering component that delegates to specific renderers

## Migration Guide

### From Plain Text to Structured Responses

1. **Update AI Service Call**:
```typescript
// Old
this.chatService.sendMessage({ message, project_id });

// New (automatic detection)
this.chatService.sendMessage({ 
  message, 
  project_id,
  use_structured_response: true 
});
```

2. **Backend automatically detects and validates**
3. **Frontend automatically renders based on content type**

### Backward Compatibility

- Plain text responses still work
- Automatic JSON detection in chat and analytics
- Fallback to text display if parsing fails

## Troubleshooting

### Common Issues

1. **"Failed to parse AI response as JSON"**
   - Ensure AI returns valid JSON
   - Check for markdown code blocks (automatically handled)
   - Verify response format matches schema

2. **"Chart must have at least one label"**
   - Ensure labels array is not empty
   - Verify dataset data length matches labels length

3. **"All table rows must match header length"**
   - Count columns in each row
   - Ensure consistency across all rows

### Debug Mode

Enable logging:
```typescript
// In content-renderer.component.ts
console.log('Parsing response:', this.response);
console.log('Validated structure:', this.structuredResponse);
```

## Future Enhancements

Potential improvements:
1. More chart types (scatter, radar, etc.)
2. Interactive tables with sorting/filtering
3. Data export functionality
4. Custom color schemes
5. Animation options
6. Accessibility improvements
7. Real-time streaming rendering
8. Caching layer for frequently used data

## Conclusion

The Rendering Module provides a robust, production-ready system for displaying rich AI responses. With comprehensive validation, error handling, and security features, it enables the DencapsBI platform to deliver engaging, interactive visualizations while maintaining data integrity and security.
