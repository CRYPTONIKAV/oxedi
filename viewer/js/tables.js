// The Tables tab: a table selector, a text filter, a virtualized grid and CSV export.

import { createVirtualList } from './virtual.js';

const ROW_HEIGHT = 28;
const CELL_WIDTH = 176;

/** RFC 4180 text for the given rows; a null cell is an empty field. */
export function toCsv(names, rows) {
  const field = (value) => {
    if (value === null || value === undefined) return '';
    return /[",\r\n]/.test(value) ? `"${value.replace(/"/g, '""')}"` : value;
  };
  const lines = [names.map(field).join(',')];
  for (const row of rows) lines.push(row.map(field).join(','));
  return `${lines.join('\r\n')}\r\n`;
}

function el(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

/**
 * @param {HTMLElement} root
 * @param {{onJump: (segment: number) => void}} options
 */
export function createTables(root, { onJump }) {
  root.textContent = '';
  const select = el('select');
  select.id = 'table-select';
  const selectLabel = el('label', null, 'Table ');
  selectLabel.htmlFor = 'table-select';
  const filter = el('input');
  filter.type = 'search';
  filter.id = 'table-filter';
  filter.placeholder = 'Filter rows';
  filter.setAttribute('aria-label', 'Filter rows');
  const count = el('span', 'muted');
  count.setAttribute('role', 'status');
  const download = el('button', null, 'Download CSV');
  download.type = 'button';
  const bar = el('div', 'toolbar');
  const group = el('span', 'group');
  group.append(selectLabel, select);
  bar.append(group, filter, count, download);
  const scroller = el('div', 'grid');
  scroller.tabIndex = 0;
  scroller.setAttribute('role', 'region');
  scroller.setAttribute('aria-label', 'Table rows');
  root.append(bar, scroller);

  let view = null;
  let name = 'file';
  let table = null;
  let haystack = [];
  let shown = [];
  let list = null;
  let segmentColumn = -1;

  function cell(value, column, row) {
    const td = el('span', 'cell');
    td.style.width = `${CELL_WIDTH}px`;
    if (value === null) {
      td.classList.add('null');
      td.textContent = '—';
      td.title = 'null';
    } else if (column === segmentColumn && value !== '') {
      const button = el('button', 'link', value);
      button.type = 'button';
      button.dataset.segment = value;
      button.title = 'Show in Text';
      td.append(button);
    } else {
      td.textContent = value;
      td.title = value;
    }
    return td;
  }

  function renderRow(position) {
    const row = shown[position];
    const div = el('div', 'row');
    div.setAttribute('role', 'row');
    if (position % 2) div.classList.add('odd');
    div.style.width = `${table.columns.length * CELL_WIDTH}px`;
    table.columns.forEach((column, c) => div.append(cell(column.values[row], c, row)));
    return div;
  }

  function applyFilter() {
    const needle = filter.value.trim().toLowerCase();
    shown = [];
    for (let row = 0; row < table.rows; row += 1) {
      if (!needle || haystack[row].includes(needle)) shown.push(row);
    }
    count.textContent = needle
      ? `${shown.length.toLocaleString()} of ${table.rows.toLocaleString()} rows`
      : `${table.rows.toLocaleString()} rows`;
    list.setCount(shown.length);
    scroller.scrollTop = 0;
    list.refresh();
  }

  function choose(tableName) {
    table = view.tables.find((t) => t.name === tableName);
    if (!table) return;
    segmentColumn = table.columns.findIndex((c) => c.name === 'segment');
    haystack = [];
    for (let row = 0; row < table.rows; row += 1) {
      haystack.push(table.columns.map((c) => c.values[row] ?? '').join('\u0001').toLowerCase());
    }
    const header = el('div', 'row head');
    header.setAttribute('role', 'row');
    header.style.width = `${table.columns.length * CELL_WIDTH}px`;
    for (const column of table.columns) {
      const th = el('span', 'cell', column.name);
      th.setAttribute('role', 'columnheader');
      th.style.width = `${CELL_WIDTH}px`;
      th.title = column.name;
      header.append(th);
    }
    if (list) list.destroy();
    list = createVirtualList({ scroller, rowHeight: ROW_HEIGHT, render: renderRow, header });
    list.setWidth(`${table.columns.length * CELL_WIDTH}px`);
    applyFilter();
  }

  let timer = 0;
  filter.addEventListener('input', () => {
    clearTimeout(timer);
    timer = setTimeout(() => table && applyFilter(), 120);
  });
  select.addEventListener('change', () => choose(select.value));
  scroller.addEventListener('click', (event) => {
    const target = event.target instanceof Element ? event.target.closest('button[data-segment]') : null;
    if (target) onJump(Number(target.dataset.segment));
  });
  download.addEventListener('click', () => {
    if (!table) return;
    const rows = shown.map((row) => table.columns.map((c) => c.values[row]));
    const csv = toCsv(table.columns.map((c) => c.name), rows);
    const url = URL.createObjectURL(new Blob([csv], { type: 'text/csv;charset=utf-8' }));
    const link = el('a');
    link.href = url;
    link.download = `${name.replace(/\.[^.]*$/, '')}-${table.name}.csv`;
    document.body.append(link);
    link.click();
    link.remove();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  });

  return {
    show(loaded) {
      view = loaded.view;
      name = loaded.name;
      select.textContent = '';
      for (const t of view.tables) {
        const option = el('option', null, `${t.name} (${t.rows.toLocaleString()})`);
        option.value = t.name;
        select.append(option);
      }
      filter.value = '';
      const first = view.tables.find((t) => t.name === 'claims') ?? view.tables[0];
      if (first) {
        select.value = first.name;
        choose(first.name);
      }
    },
    /** Called when the tab becomes visible: the grid can only measure itself then. */
    onShow() {
      if (list) list.refresh();
    },
    clear() {
      view = null;
      table = null;
      if (list) list.destroy();
      list = null;
      select.textContent = '';
      scroller.textContent = '';
      count.textContent = '';
    },
  };
}
