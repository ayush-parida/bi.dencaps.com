import { Injectable } from '@angular/core';
import { StructuredResponse, RenderItem } from './rendering.models';

@Injectable({
  providedIn: 'root'
})
export class RenderingService {
  
  /**
   * Parses and validates a structured response from the backend
   */
  parseStructuredResponse(response: any): StructuredResponse {
    if (!response || typeof response !== 'object') {
      throw new Error('Invalid response: must be an object');
    }

    if (!Array.isArray(response.items)) {
      throw new Error('Invalid response: items must be an array');
    }

    if (response.items.length === 0) {
      throw new Error('Invalid response: must contain at least one item');
    }

    // Validate each item
    response.items.forEach((item: any, index: number) => {
      this.validateItem(item, index);
    });

    return response as StructuredResponse;
  }

  /**
   * Validates a single render item
   */
  private validateItem(item: any, index: number): void {
    if (!item || typeof item !== 'object') {
      throw new Error(`Invalid item at index ${index}: must be an object`);
    }

    if (!item.type) {
      throw new Error(`Invalid item at index ${index}: missing type`);
    }

    const validTypes = ['text', 'chart', 'equation', 'table', 'dataset'];
    if (!validTypes.includes(item.type)) {
      throw new Error(`Invalid item at index ${index}: type must be one of ${validTypes.join(', ')}`);
    }

    switch (item.type) {
      case 'text':
        this.validateTextContent(item, index);
        break;
      case 'chart':
        this.validateChartContent(item, index);
        break;
      case 'equation':
        this.validateEquationContent(item, index);
        break;
      case 'table':
        this.validateTableContent(item, index);
        break;
      case 'dataset':
        this.validateDatasetContent(item, index);
        break;
    }
  }

  private validateTextContent(item: any, index: number): void {
    if (item.content == null || typeof item.content !== 'string') {
      throw new Error(`Invalid text content at index ${index}: content must be a string`);
    }
  }

  private validateChartContent(item: any, index: number): void {
    if (!item.data || typeof item.data !== 'object') {
      throw new Error(`Invalid chart content at index ${index}: data must be an object`);
    }

    const validChartTypes = ['bar', 'line', 'pie'];
    if (!validChartTypes.includes(item.data.chart_type)) {
      throw new Error(`Invalid chart content at index ${index}: chart_type must be one of ${validChartTypes.join(', ')}`);
    }

    if (!Array.isArray(item.data.labels) || item.data.labels.length === 0) {
      throw new Error(`Invalid chart content at index ${index}: labels must be a non-empty array`);
    }

    if (!Array.isArray(item.data.datasets) || item.data.datasets.length === 0) {
      throw new Error(`Invalid chart content at index ${index}: datasets must be a non-empty array`);
    }

    item.data.datasets.forEach((dataset: any, dsIndex: number) => {
      if (!Array.isArray(dataset.data) || dataset.data.length !== item.data.labels.length) {
        throw new Error(`Invalid chart content at index ${index}: dataset ${dsIndex} data length must match labels length`);
      }
    });
  }

  private validateEquationContent(item: any, index: number): void {
    if (item.latex == null || typeof item.latex !== 'string') {
      throw new Error(`Invalid equation content at index ${index}: latex must be a string`);
    }
    if (item.latex.trim() === '') {
      throw new Error(`Invalid equation content at index ${index}: latex cannot be empty`);
    }
  }

  private validateTableContent(item: any, index: number): void {
    if (!item.data || typeof item.data !== 'object') {
      throw new Error(`Invalid table content at index ${index}: data must be an object`);
    }

    if (!Array.isArray(item.data.headers) || item.data.headers.length === 0) {
      throw new Error(`Invalid table content at index ${index}: headers must be a non-empty array`);
    }

    if (!Array.isArray(item.data.rows)) {
      throw new Error(`Invalid table content at index ${index}: rows must be an array`);
    }

    item.data.rows.forEach((row: any, rowIndex: number) => {
      if (!Array.isArray(row) || row.length !== item.data.headers.length) {
        throw new Error(`Invalid table content at index ${index}: row ${rowIndex} length must match headers length`);
      }
    });
  }

  private validateDatasetContent(item: any, index: number): void {
    if (!item.data || typeof item.data !== 'object') {
      throw new Error(`Invalid dataset content at index ${index}: data must be an object`);
    }

    if (item.data.name == null || typeof item.data.name !== 'string') {
      throw new Error(`Invalid dataset content at index ${index}: name must be a string`);
    }

    if (item.data.name.trim() === '') {
      throw new Error(`Invalid dataset content at index ${index}: name cannot be empty`);
    }

    if (!Array.isArray(item.data.columns) || item.data.columns.length === 0) {
      throw new Error(`Invalid dataset content at index ${index}: columns must be a non-empty array`);
    }

    if (!Array.isArray(item.data.rows)) {
      throw new Error(`Invalid dataset content at index ${index}: rows must be an array`);
    }

    item.data.rows.forEach((row: any, rowIndex: number) => {
      if (!Array.isArray(row) || row.length !== item.data.columns.length) {
        throw new Error(`Invalid dataset content at index ${index}: row ${rowIndex} length must match columns length`);
      }
    });
  }
}
