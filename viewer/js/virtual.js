// A fixed-row-height virtual list: only the rows in (and just around) the
// viewport exist in the DOM. The scroller holds one sizer; rows are placed
// absolutely inside it.

const OVERSCAN = 6;

/**
 * @param {object} options
 * @param {HTMLElement} options.scroller  element with overflow: auto and a height
 * @param {number} options.rowHeight      pixels
 * @param {(index: number) => HTMLElement} options.render  builds the row for an index
 * @param {HTMLElement} [options.header]  sticky header placed before the sizer
 */
export function createVirtualList({ scroller, rowHeight, render, header }) {
  const sizer = document.createElement('div');
  sizer.className = 'v-sizer';
  scroller.textContent = '';
  if (header) scroller.append(header);
  scroller.append(sizer);

  let count = 0;
  let rows = new Map();
  let pending = false;

  function paint() {
    pending = false;
    const top = Math.max(0, scroller.scrollTop - (header ? header.offsetHeight : 0));
    const height = scroller.clientHeight;
    const first = Math.max(0, Math.floor(top / rowHeight) - OVERSCAN);
    const last = Math.min(count - 1, Math.ceil((top + height) / rowHeight) + OVERSCAN);
    for (const [index, el] of rows) {
      if (index < first || index > last) {
        el.remove();
        rows.delete(index);
      }
    }
    for (let index = first; index <= last; index += 1) {
      if (rows.has(index)) continue;
      const el = render(index);
      el.style.position = 'absolute';
      el.style.top = '0';
      el.style.left = '0';
      el.style.height = `${rowHeight}px`;
      el.style.transform = `translateY(${index * rowHeight}px)`;
      sizer.append(el);
      rows.set(index, el);
    }
  }

  function schedule() {
    if (pending) return;
    pending = true;
    requestAnimationFrame(paint);
  }

  scroller.addEventListener('scroll', schedule, { passive: true });
  const observer = new ResizeObserver(schedule);
  observer.observe(scroller);

  return {
    /** Replaces the row count and drops every rendered row. */
    setCount(next) {
      count = next;
      sizer.style.height = `${count * rowHeight}px`;
      for (const el of rows.values()) el.remove();
      rows = new Map();
      paint();
    },
    /** Re-renders what is visible, after the data behind `render` changed. */
    refresh() {
      for (const el of rows.values()) el.remove();
      rows = new Map();
      paint();
    },
    /** Sets the width rows can use (for horizontal scrolling). */
    setWidth(width) {
      sizer.style.width = width;
    },
    /** Scrolls so that the row is roughly in the middle of the viewport. */
    scrollToIndex(index) {
      const offset = header ? header.offsetHeight : 0;
      const target = index * rowHeight + offset - (scroller.clientHeight - rowHeight) / 2;
      scroller.scrollTop = Math.max(0, target);
      paint();
    },
    get count() {
      return count;
    },
    destroy() {
      observer.disconnect();
      scroller.removeEventListener('scroll', schedule);
    },
  };
}
