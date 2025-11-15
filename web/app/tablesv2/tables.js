function tablesApp() {
  return {
    data: [],
    selectedCell: null,
    cols: 26,
    rows: 100,
    visibleRows: 30,
    rowOffset: 0,
    
    init() {
      this.data = this.generateMockData(this.rows, this.cols);
      this.setupEventListeners();
      this.updateStats();
    },

    generateMockData(rows, cols) {
      const data = [];
      const products = ['Laptop', 'Mouse', 'Keyboard', 'Monitor', 'Headphones', 'Webcam', 'Desk', 'Chair'];
      const regions = ['North', 'South', 'East', 'West'];
      
      for (let i = 0; i < rows; i++) {
        const row = {};
        for (let j = 0; j < cols; j++) {
          const col = this.getColumnName(j);
          if (i === 0) {
            if (j === 0) row[col] = 'Product';
            else if (j === 1) row[col] = 'Region';
            else if (j === 2) row[col] = 'Q1';
            else if (j === 3) row[col] = 'Q2';
            else if (j === 4) row[col] = 'Q3';
            else if (j === 5) row[col] = 'Q4';
            else if (j === 6) row[col] = 'Total';
            else row[col] = `Col ${col}`;
          } else {
            if (j === 0) row[col] = products[i % products.length];
            else if (j === 1) row[col] = regions[i % regions.length];
            else if (j >= 2 && j <= 5) row[col] = Math.floor(Math.random() * 10000) + 1000;
            else if (j === 6) {
              row[col] = `=C${i+1}+D${i+1}+E${i+1}+F${i+1}`;
            }
            else row[col] = Math.random() > 0.5 ? Math.floor(Math.random() * 1000) : '';
          }
        }
        data.push(row);
      }
      return data;
    },

    getColumnName(index) {
      let name = '';
      while (index >= 0) {
        name = String.fromCharCode(65 + (index % 26)) + name;
        index = Math.floor(index / 26) - 1;
      }
      return name;
    },

    setupEventListeners() {
      // Will be replaced with Alpine.js directives
    },

    selectCell(cell) {
      if (this.selectedCell) {
        this.selectedCell.classList.remove('selected');
      }
      this.selectedCell = cell;
      cell.classList.add('selected');
      
      const cellRef = cell.dataset.cell;
      document.getElementById('cellRef').textContent = cellRef;
      document.getElementById('selectedCell').textContent = cellRef;
      
      const row = parseInt(cell.dataset.row);
      const col = this.getColumnName(parseInt(cell.dataset.col));
      const value = this.data[row][col] || '';
      document.getElementById('formulaInput').value = value;
    },

    calculateCell(value, row, col) {
      if (typeof value === 'string' && value.startsWith('=')) {
        try {
          const formula = value.substring(1).toUpperCase();
          
          if (formula.includes('SUM')) {
            const match = formula.match(/SUM\(([A-Z]+\d+):([A-Z]+\d+)\)/);
            if (match) {
              const sum = this.calculateRange(match[1], match[2], 'sum');
              return sum.toFixed(2);
            }
          }
          
          if (formula.includes('AVERAGE')) {
            const match = formula.match(/AVERAGE\(([A-Z]+\d+):([A-Z]+\d+)\)/);
            if (match) {
              const avg = this.calculateRange(match[1], match[2], 'avg');
              return avg.toFixed(2);
            }
          }
          
          let expression = formula;
          const cellRefs = expression.match(/[A-Z]+\d+/g);
          if (cellRefs) {
            cellRefs.forEach(ref => {
              const val = this.getCellValue(ref);
              expression = expression.replace(ref, val);
            });
            return eval(expression).toFixed(2);
          }
        } catch (e) {
          return '#ERROR';
        }
      }
      return value;
    },

    getCellValue(cellRef) {
      const col = cellRef.match(/[A-Z]+/)[0];
      const row = parseInt(cellRef.match(/\d+/)[0]) - 1;
      const value = this.data[row][col];
      
      if (typeof value === 'string' && value.startsWith('=')) {
        return this.calculateCell(value, row, this.getColIndex(col));
      }
      return parseFloat(value) || 0;
    },

    getColIndex(colName) {
      let index = 0;
      for (let i = 0; i < colName.length; i++) {
        index = index * 26 + (colName.charCodeAt(i) - 64);
      }
      return index - 1;
    },

    calculateRange(start, end, operation) {
      const startCol = start.match(/[A-Z]+/)[0];
      const startRow = parseInt(start.match(/\d+/)[0]) - 1;
      const endCol = end.match(/[A-Z]+/)[0];
      const endRow = parseInt(end.match(/\d+/)[0]) - 1;
      
      let values = [];
      for (let r = startRow; r <= endRow; r++) {
        for (let c = this.getColIndex(startCol); c <= this.getColIndex(endCol); c++) {
          const col = this.getColumnName(c);
          const val = parseFloat(this.data[r][col]) || 0;
          values.push(val);
        }
      }
      
      if (operation === 'sum') {
        return values.reduce((a, b) => a + b, 0);
      } else if (operation === 'avg') {
        return values.reduce((a, b) => a + b, 0) / values.length;
      }
      return 0;
    },

    updateCellValue(value) {
      if (!this.selectedCell) return;
      
      const row = parseInt(this.selectedCell.dataset.row);
      const col = this.getColumnName(parseInt(this.selectedCell.dataset.col));
      
      this.data[row][col] = value;
      this.renderTable();
      
      const newCell = document.querySelector(`td[data-row="${row}"][data-col="${this.selectedCell.dataset.col}"]`);
      if (newCell) this.selectCell(newCell);
    },

    renderTable() {
      const thead = document.getElementById('tableHead');
      const tbody = document.getElementById('tableBody');
      
      let headerHTML = '<tr><th></th>';
      for (let i = 0; i < this.cols; i++) {
        headerHTML += `<th>${this.getColumnName(i)}</th>`;
      }
      headerHTML += '</tr>';
      thead.innerHTML = headerHTML;

      let bodyHTML = '';
      const endRow = Math.min(this.rowOffset + this.visibleRows, this.rows);
      
      for (let i = this.rowOffset; i < endRow; i++) {
        bodyHTML += `<tr><th>${i + 1}</th>`;
        for (let j = 0; j < this.cols; j++) {
          const col = this.getColumnName(j);
          const value = this.data[i][col] || '';
          const displayValue = this.calculateCell(value, i, j);
          bodyHTML += `<td @click="selectCell($el)" 
                          data-row="${i}" 
                          data-col="${j}" 
                          data-cell="${col}${i+1}"
                          :class="{ 'selected': selectedCell === $el }">
                          ${displayValue}
                      </td>`;
        }
        bodyHTML += '</tr>';
      }
      tbody.innerHTML = bodyHTML;
    },

    updateStats() {
      document.getElementById('rowCount').textContent = this.rows;
      document.getElementById('colCount').textContent = this.cols;
    },

    // Toolbar actions
    addRow() {
      const newRow = {};
      for (let i = 0; i < this.cols; i++) {
        newRow[this.getColumnName(i)] = '';
      }
      this.data.push(newRow);
      this.rows++;
      this.renderTable();
      this.updateStats();
    },

    addColumn() {
      const newCol = this.getColumnName(this.cols);
      this.data.forEach(row => row[newCol] = '');
      this.cols++;
      this.renderTable();
      this.updateStats();
    },

    deleteRow() {
      if (this.selectedCell && this.rows > 1) {
        const row = parseInt(this.selectedCell.dataset.row);
        this.data.splice(row, 1);
        this.rows--;
        this.renderTable();
        this.updateStats();
      }
    },

    deleteColumn() {
      if (this.selectedCell && this.cols > 1) {
        const col = this.getColumnName(parseInt(this.selectedCell.dataset.col));
        this.data.forEach(row => delete row[col]);
        this.cols--;
        this.renderTable();
        this.updateStats();
      }
    },

    sort() {
      if (this.selectedCell) {
        const col = this.getColumnName(parseInt(this.selectedCell.dataset.col));
        const header = this.data[0];
        const dataRows = this.data.slice(1);
        
        dataRows.sort((a, b) => {
          const aVal = a[col] || '';
          const bVal = b[col] || '';
          return aVal.toString().localeCompare(bVal.toString());
        });
        
        this.data = [header, ...dataRows];
        this.renderTable();
      }
    },

    sum() {
      if (this.selectedCell) {
        const col = this.getColumnName(parseInt(this.selectedCell.dataset.col));
        this.formulaInputValue = `=SUM(${col}2:${col}${this.rows})`;
      }
    },

    average() {
      if (this.selectedCell) {
        const col = this.getColumnName(parseInt(this.selectedCell.dataset.col));
        this.formulaInputValue = `=AVERAGE(${col}2:${col}${this.rows})`;
      }
    },

    exportData() {
      const csv = this.data.map(row => {
        return Object.values(row).join(',');
      }).join('\n');
      
      const blob = new Blob([csv], { type: 'text/csv' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'tables_export.csv';
      a.click();
    }
  };
}
