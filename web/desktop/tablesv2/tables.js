// Tables module JavaScript
document.addEventListener('alpine:init', () => {
  Alpine.data('tablesApp', () => ({
    data: [],
    selectedCell: null,
    cols: 26,
    rows: 100,
    init() {
      this.data = this.generateMockData(this.rows, this.cols);
      this.renderTable();
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

    selectCell(cell) {
      if (this.selectedCell) {
        this.selectedCell.classList.remove('selected');
      }
      this.selectedCell = cell;
      cell.classList.add('selected');
      
      const cellRef = cell.dataset.cell;
      document.getElementById('cellRef').textContent = cellRef;
      
      const row = parseInt(cell.dataset.row);
      const col = this.getColumnName(parseInt(cell.dataset.col));
      const value = this.data[row][col] || '';
      document.getElementById('formulaInput').value = value;
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
      for (let i = 0; i < this.rows; i++) {
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

    // Toolbar actions
    addRow() {
      const newRow = {};
      for (let i = 0; i < this.cols; i++) {
        newRow[this.getColumnName(i)] = '';
      }
      this.data.push(newRow);
      this.rows++;
      this.renderTable();
    },

    addColumn() {
      const newCol = this.getColumnName(this.cols);
      this.data.forEach(row => row[newCol] = '');
      this.cols++;
      this.renderTable();
    },

    deleteRow() {
      if (this.selectedCell && this.rows > 1) {
        const row = parseInt(this.selectedCell.dataset.row);
        this.data.splice(row, 1);
        this.rows--;
        this.renderTable();
      }
    },

    deleteColumn() {
      if (this.selectedCell && this.cols > 1) {
        const col = this.getColumnName(parseInt(this.selectedCell.dataset.col));
        this.data.forEach(row => delete row[col]);
        this.cols--;
        this.renderTable();
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
  }));
});
