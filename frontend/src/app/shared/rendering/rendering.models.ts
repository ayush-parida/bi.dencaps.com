export type RenderContentType = 'text' | 'chart' | 'equation' | 'table' | 'dataset';

export interface RenderContent {
  type: RenderContentType;
}

export interface TextContent extends RenderContent {
  type: 'text';
  content: string;
}

export interface ChartContent extends RenderContent {
  type: 'chart';
  data: ChartData;
}

export interface EquationContent extends RenderContent {
  type: 'equation';
  latex: string;
  display?: boolean;
}

export interface TableContent extends RenderContent {
  type: 'table';
  data: TableData;
}

export interface DatasetContent extends RenderContent {
  type: 'dataset';
  data: DatasetData;
}

export type RenderItem = TextContent | ChartContent | EquationContent | TableContent | DatasetContent;

export interface StructuredResponse {
  items: RenderItem[];
}

export interface ChartData {
  chart_type: 'bar' | 'line' | 'pie';
  title?: string;
  labels: string[];
  datasets: ChartDataset[];
}

export interface ChartDataset {
  label: string;
  data: number[];
  background_color?: string;
  border_color?: string;
}

export interface TableData {
  headers: string[];
  rows: string[][];
}

export interface DatasetData {
  name: string;
  description?: string;
  columns: ColumnInfo[];
  rows: any[][];
}

export interface ColumnInfo {
  name: string;
  data_type: string;
}
