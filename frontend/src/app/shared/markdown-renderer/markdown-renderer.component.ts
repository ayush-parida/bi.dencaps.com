import { Component, Input, OnChanges, SimpleChanges, AfterViewChecked, ElementRef, ViewChild } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DomSanitizer, SafeHtml } from '@angular/platform-browser';

@Component({
  selector: 'app-markdown-renderer',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="markdown-content" #markdownContainer [innerHTML]="renderedContent"></div>
  `,
  styles: [`
    .markdown-content {
      line-height: 1.6;
      color: #333;
    }

    :host ::ng-deep .markdown-content h1,
    :host ::ng-deep .markdown-content h2,
    :host ::ng-deep .markdown-content h3,
    :host ::ng-deep .markdown-content h4,
    :host ::ng-deep .markdown-content h5,
    :host ::ng-deep .markdown-content h6 {
      margin-top: 1rem;
      margin-bottom: 0.5rem;
      font-weight: 600;
      line-height: 1.25;
    }

    :host ::ng-deep .markdown-content h1 { font-size: 1.5em; border-bottom: 1px solid #e1e8ed; padding-bottom: 0.3em; }
    :host ::ng-deep .markdown-content h2 { font-size: 1.3em; border-bottom: 1px solid #e1e8ed; padding-bottom: 0.3em; }
    :host ::ng-deep .markdown-content h3 { font-size: 1.15em; }
    :host ::ng-deep .markdown-content h4 { font-size: 1em; }

    /* Accordion styles */
    :host ::ng-deep .markdown-content .accordion-section {
      margin: 0.5rem 0;
      border: 1px solid #e1e8ed;
      border-radius: 8px;
      overflow: hidden;
    }

    :host ::ng-deep .markdown-content .accordion-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0.75rem 1rem;
      background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 100%);
      cursor: pointer;
      user-select: none;
      transition: background 0.2s ease;
      border: none;
      margin: 0;
    }

    :host ::ng-deep .markdown-content .accordion-header:hover {
      background: linear-gradient(135deg, #e9ecef 0%, #dee2e6 100%);
    }

    :host ::ng-deep .markdown-content .accordion-header h2,
    :host ::ng-deep .markdown-content .accordion-header h3,
    :host ::ng-deep .markdown-content .accordion-header h4,
    :host ::ng-deep .markdown-content .accordion-header h5,
    :host ::ng-deep .markdown-content .accordion-header h6 {
      margin: 0;
      border: none;
      padding: 0;
      flex: 1;
    }

    :host ::ng-deep .markdown-content .accordion-icon {
      font-size: 0.8em;
      transition: transform 0.3s ease;
      color: #667eea;
    }

    :host ::ng-deep .markdown-content .accordion-section.collapsed .accordion-icon {
      transform: rotate(-90deg);
    }

    :host ::ng-deep .markdown-content .accordion-content {
      padding: 1rem;
      background: #fff;
      transition: max-height 0.3s ease, padding 0.3s ease, opacity 0.3s ease;
      overflow: hidden;
    }

    :host ::ng-deep .markdown-content .accordion-section.collapsed .accordion-content {
      max-height: 0;
      padding: 0 1rem;
      opacity: 0;
    }

    :host ::ng-deep .markdown-content .accordion-section:not(.collapsed) .accordion-content {
      max-height: 5000px;
      opacity: 1;
    }

    :host ::ng-deep .markdown-content p {
      margin: 0.5rem 0;
    }

    :host ::ng-deep .markdown-content ul,
    :host ::ng-deep .markdown-content ol {
      margin: 0.5rem 0;
      padding-left: 1.5rem;
    }

    :host ::ng-deep .markdown-content li {
      margin: 0.25rem 0;
    }

    :host ::ng-deep .markdown-content code {
      background-color: #f6f8fa;
      padding: 0.2em 0.4em;
      border-radius: 4px;
      font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
      font-size: 0.9em;
      color: #d63384;
    }

    :host ::ng-deep .markdown-content pre {
      background-color: #1e1e1e;
      color: #d4d4d4;
      padding: 1rem;
      border-radius: 8px;
      overflow-x: auto;
      margin: 0.75rem 0;
      font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
      font-size: 0.9em;
      line-height: 1.5;
    }

    :host ::ng-deep .markdown-content pre code {
      background: transparent;
      padding: 0;
      color: inherit;
      font-size: inherit;
    }

    :host ::ng-deep .markdown-content blockquote {
      border-left: 4px solid #667eea;
      margin: 0.75rem 0;
      padding: 0.5rem 1rem;
      background-color: #f8f9fa;
      color: #6c757d;
    }

    :host ::ng-deep .markdown-content table {
      border-collapse: collapse;
      width: 100%;
      margin: 0.75rem 0;
    }

    :host ::ng-deep .markdown-content th,
    :host ::ng-deep .markdown-content td {
      border: 1px solid #e1e8ed;
      padding: 0.5rem 0.75rem;
      text-align: left;
    }

    :host ::ng-deep .markdown-content th {
      background-color: #f6f8fa;
      font-weight: 600;
    }

    :host ::ng-deep .markdown-content tr:nth-child(even) {
      background-color: #f8f9fa;
    }

    :host ::ng-deep .markdown-content a {
      color: #667eea;
      text-decoration: none;
    }

    :host ::ng-deep .markdown-content a:hover {
      text-decoration: underline;
    }

    :host ::ng-deep .markdown-content hr {
      border: none;
      border-top: 1px solid #e1e8ed;
      margin: 1rem 0;
    }

    :host ::ng-deep .markdown-content img {
      max-width: 100%;
      height: auto;
      border-radius: 4px;
    }

    :host ::ng-deep .markdown-content strong {
      font-weight: 600;
    }

    :host ::ng-deep .markdown-content em {
      font-style: italic;
    }
  `]
})
export class MarkdownRendererComponent implements OnChanges, AfterViewChecked {
  @Input() content: string = '';
  @ViewChild('markdownContainer') markdownContainer!: ElementRef;
  
  renderedContent: SafeHtml = '';
  private needsEventBinding = false;

  constructor(private sanitizer: DomSanitizer) {}

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['content']) {
      this.renderMarkdown();
      this.needsEventBinding = true;
    }
  }

  ngAfterViewChecked(): void {
    if (this.needsEventBinding && this.markdownContainer) {
      this.bindAccordionEvents();
      this.needsEventBinding = false;
    }
  }

  private bindAccordionEvents(): void {
    const container = this.markdownContainer.nativeElement;
    const headers = container.querySelectorAll('.accordion-header');
    
    headers.forEach((header: HTMLElement) => {
      header.addEventListener('click', () => {
        const section = header.closest('.accordion-section');
        if (section) {
          section.classList.toggle('collapsed');
        }
      });
    });
  }

  private renderMarkdown(): void {
    if (!this.content) {
      this.renderedContent = '';
      return;
    }

    // Simple markdown parsing without external dependencies
    let html = this.content;
    
    // Escape HTML first (but preserve < and > in code blocks later)
    html = html.replace(/&/g, '&amp;');

    // Code blocks (triple backticks) - process first to protect content
    const codeBlocks: string[] = [];
    html = html.replace(/```(\w*)\n?([\s\S]*?)```/g, (_, lang, code) => {
      const escaped = code.replace(/</g, '&lt;').replace(/>/g, '&gt;');
      codeBlocks.push(`<pre><code class="language-${lang}">${escaped.trim()}</code></pre>`);
      return `\n%%CODEBLOCK${codeBlocks.length - 1}%%\n`;
    });

    // Escape remaining < and >
    html = html.replace(/</g, '&lt;').replace(/>/g, '&gt;');

    // Inline code (single backticks)
    html = html.replace(/`([^`]+)`/g, '<code>$1</code>');

    // Tables - process before other elements
    html = this.parseTable(html);

    // Headers
    html = html.replace(/^######\s+(.+)$/gm, '<h6>$1</h6>');
    html = html.replace(/^#####\s+(.+)$/gm, '<h5>$1</h5>');
    html = html.replace(/^####\s+(.+)$/gm, '<h4>$1</h4>');
    html = html.replace(/^###\s+(.+)$/gm, '<h3>$1</h3>');
    html = html.replace(/^##\s+(.+)$/gm, '<h2>$1</h2>');
    html = html.replace(/^#\s+(.+)$/gm, '<h1>$1</h1>');

    // Bold and Italic
    html = html.replace(/\*\*\*(.+?)\*\*\*/g, '<strong><em>$1</em></strong>');
    html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
    html = html.replace(/\*(.+?)\*/g, '<em>$1</em>');
    html = html.replace(/___(.+?)___/g, '<strong><em>$1</em></strong>');
    html = html.replace(/__(.+?)__/g, '<strong>$1</strong>');
    html = html.replace(/_(.+?)_/g, '<em>$1</em>');

    // Blockquotes
    html = html.replace(/^&gt;\s+(.+)$/gm, '<blockquote>$1</blockquote>');

    // Horizontal rules
    html = html.replace(/^---$/gm, '<hr>');
    html = html.replace(/^\*\*\*$/gm, '<hr>');

    // Unordered lists
    html = html.replace(/^[\*\-]\s+(.+)$/gm, '<li>$1</li>');
    html = html.replace(/(<li>.*<\/li>\n?)+/g, '<ul>$&</ul>');

    // Ordered lists
    html = html.replace(/^\d+\.\s+(.+)$/gm, '<li>$1</li>');

    // Links
    html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noopener">$1</a>');

    // Paragraphs (wrap remaining text blocks)
    html = html.split('\n\n').map(block => {
      block = block.trim();
      if (!block) return '';
      // Don't wrap if it's already an HTML element or code block placeholder
      if (block.match(/^<(h[1-6]|ul|ol|li|pre|blockquote|hr|table)/) || block.match(/^%%CODEBLOCK\d+%%$/)) {
        return block;
      }
      // Replace single newlines with <br>
      block = block.replace(/\n/g, '<br>');
      return `<p>${block}</p>`;
    }).join('\n');

    // Clean up nested paragraph tags
    html = html.replace(/<p><(h[1-6]|ul|ol|pre|blockquote|hr|table)/g, '<$1');
    html = html.replace(/<\/(h[1-6]|ul|ol|pre|blockquote|hr|table)><\/p>/g, '</$1>');

    // Restore code blocks
    codeBlocks.forEach((block, index) => {
      html = html.replace(`%%CODEBLOCK${index}%%`, block);
    });

    // Convert escaped <br> back to actual <br> tags
    html = html.replace(/&lt;br&gt;/gi, '<br>');

    // Wrap sections (h2, h3, h4) in accordions
    html = this.wrapInAccordions(html);

    this.renderedContent = this.sanitizer.bypassSecurityTrustHtml(html);
  }

  private wrapInAccordions(html: string): string {
    // Parse HTML into sections based on h1, h2, h3 headers only
    const tempDiv = document.createElement('div');
    tempDiv.innerHTML = html;
    
    const result: string[] = [];
    let currentSection: { header: Element | null; content: string[] } | null = null;
    let sectionIndex = 0;
    
    const children = Array.from(tempDiv.childNodes);
    
    for (const node of children) {
      if (node.nodeType === Node.ELEMENT_NODE) {
        const element = node as Element;
        const tagName = element.tagName?.toLowerCase();
        
        // Check if this is an accordion-worthy header (h1, h2, h3 only)
        if (['h1', 'h2', 'h3'].includes(tagName)) {
          // Close previous section if exists
          if (currentSection) {
            result.push(this.createAccordionHtml(currentSection.header!, currentSection.content, sectionIndex++));
          }
          // Start new section
          currentSection = { header: element, content: [] };
        } else if (currentSection) {
          // Add to current section content
          currentSection.content.push(element.outerHTML);
        } else {
          // No section yet, add directly to result
          result.push(element.outerHTML);
        }
      } else if (node.nodeType === Node.TEXT_NODE) {
        const text = node.textContent?.trim();
        if (text) {
          if (currentSection) {
            currentSection.content.push(text);
          } else {
            result.push(text);
          }
        }
      }
    }
    
    // Close last section
    if (currentSection) {
      result.push(this.createAccordionHtml(currentSection.header!, currentSection.content, sectionIndex));
    }
    
    return result.join('\n');
  }

  private createAccordionHtml(header: Element, content: string[], index: number): string {
    const headerTag = header.tagName.toLowerCase();
    const headerContent = header.innerHTML;
    
    return `
      <div class="accordion-section" data-accordion="${index}">
        <div class="accordion-header">
          <${headerTag}>${headerContent}</${headerTag}>
          <span class="accordion-icon">▼</span>
        </div>
        <div class="accordion-content">
          ${content.join('\n')}
        </div>
      </div>
    `;
  }

  private parseTable(html: string): string {
    // Match markdown tables - lines starting with | (with or without trailing |)
    const lines = html.split('\n');
    const result: string[] = [];
    let i = 0;
    
    while (i < lines.length) {
      const line = lines[i];
      
      // Check if this line starts a table (starts with |)
      if (line.trim().startsWith('|')) {
        const tableLines: string[] = [];
        
        // Collect all consecutive lines that look like table rows
        while (i < lines.length && lines[i].trim().startsWith('|')) {
          tableLines.push(lines[i]);
          i++;
        }
        
        // Need at least 2 lines (header + separator) to be a valid table
        if (tableLines.length >= 2) {
          const separatorRow = tableLines[1];
          // Check if second row is separator (contains dashes)
          if (separatorRow.match(/^\|[\s\-:|]+$/)) {
            let tableHtml = '<table>';
            
            // Header row
            const headerCells = this.parseTableRow(tableLines[0]);
            tableHtml += '<thead><tr>';
            headerCells.forEach((cell: string) => {
              tableHtml += `<th>${cell}</th>`;
            });
            tableHtml += '</tr></thead>';
            
            // Body rows
            tableHtml += '<tbody>';
            for (let j = 2; j < tableLines.length; j++) {
              const cells = this.parseTableRow(tableLines[j]);
              if (cells.length > 0) {
                tableHtml += '<tr>';
                cells.forEach((cell: string) => {
                  tableHtml += `<td>${cell}</td>`;
                });
                tableHtml += '</tr>';
              }
            }
            tableHtml += '</tbody></table>';
            
            result.push(tableHtml);
            continue;
          }
        }
        
        // Not a valid table, add lines back
        result.push(...tableLines);
        continue;
      }
      
      result.push(line);
      i++;
    }
    
    return result.join('\n');
  }

  private parseTableRow(row: string): string[] {
    // Remove leading/trailing pipes and split
    let cells = row.trim();
    if (cells.startsWith('|')) cells = cells.substring(1);
    if (cells.endsWith('|')) cells = cells.substring(0, cells.length - 1);
    
    return cells.split('|').map((cell: string) => {
      let content = cell.trim();
      // Handle <br> tags that were escaped
      content = content.replace(/&lt;br&gt;/gi, '<br>');
      content = content.replace(/<br>/gi, '<br>');
      return content;
    });
  }
}
