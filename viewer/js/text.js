// The Text tab: the file one segment per line, virtualized, decoded one
// segment at a time from the original bytes.

import { createVirtualList } from './virtual.js';
import { decode, elementRange, loopPath, visible } from './bytes.js';

const ROW_HEIGHT = 22;

function el(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

export function createText(root) {
  root.textContent = '';
  const scroller = el('div', 'textview');
  scroller.tabIndex = 0;
  scroller.setAttribute('role', 'region');
  scroller.setAttribute('aria-label', 'Original file, one segment per line');
  root.append(scroller);

  let loaded = null;
  let list = null;
  // The selected segment, and the byte range highlighted inside it.
  let selected = null;
  let range = null;

  // With a selection, the list has one extra row right after the selected
  // segment that shows its loop path.
  const rowCount = () => loaded.view.segments.length + (selected === null ? 0 : 1);
  const itemAt = (row) => {
    if (selected === null || row <= selected) return { segment: row };
    if (row === selected + 1) return { path: true };
    return { segment: row - 1 };
  };

  function lineText(index) {
    const { bytes, view } = loaded;
    const { start, end } = view.segments[index];
    const line = el('span', 'raw');
    if (index === selected && range && range[0] >= start && range[1] <= end) {
      const [from, to] = range;
      line.append(visible(decode(bytes, start, from)));
      const mark = el('mark', null, visible(decode(bytes, from, to)));
      line.append(mark, visible(decode(bytes, to, end)));
    } else {
      line.textContent = visible(decode(bytes, start, end));
    }
    return line;
  }

  function render(row) {
    const item = itemAt(row);
    const div = el('div', 'line');
    if (item.path) {
      div.classList.add('path');
      const path = loopPath(loaded.view, selected);
      div.textContent = path.length ? path.join(' › ') : '(outside every loop)';
      return div;
    }
    div.dataset.segment = String(item.segment);
    if (item.segment === selected) div.classList.add('current');
    div.append(el('span', 'no', String(item.segment)), lineText(item.segment));
    return div;
  }

  function mount() {
    if (list) list.destroy();
    list = createVirtualList({ scroller, rowHeight: ROW_HEIGHT, render });
    let longest = 0;
    for (const s of loaded.view.segments) longest = Math.max(longest, s.end - s.start);
    list.setWidth(`calc(${longest}ch + 8ch)`);
    list.setCount(rowCount());
  }

  return {
    show(next) {
      loaded = next;
      selected = null;
      range = null;
      scroller.scrollTop = 0;
      mount();
    },
    /** Highlights a segment, and one element or component of it when given (1-based; 0 is the id). */
    showSegment(index, element, component) {
      if (!loaded || !loaded.view.segments[index]) return;
      selected = index;
      range = elementRange(loaded.bytes, loaded.view, index, element, component);
      list.setCount(rowCount());
      list.scrollToIndex(index);
      scroller.focus({ preventScroll: true });
    },
    onShow() {
      if (list) list.refresh();
    },
    clear() {
      loaded = null;
      selected = null;
      range = null;
      if (list) list.destroy();
      list = null;
      scroller.textContent = '';
    },
  };
}
