// A fixed-row-height virtual list: only the rows in (and just around) the
// viewport exist in the DOM. The scroller holds one sizer; rows are placed
// absolutely inside it.

const OVERSCAN = 6;
const MAX_HEIGHT = 10_000_000;

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
  let scale = 1;

  // Browsers cap an element's height; past MAX_HEIGHT the sizer stays at the
  // cap and the scroll position is mapped proportionally onto the full list.
  const headerHeight = () => (header ? header.offsetHeight : 0);
  const scrollable = () => Math.max(0, sizer.offsetHeight + headerHeight() - scroller.clientHeight);
  const fullScrollable = () => Math.max(0, count * rowHeight + headerHeight() - scroller.clientHeight);
  // The list offset the viewport top shows.
  const virtualTop = () => (scale === 1 ? scroller.scrollTop : scroller.scrollTop * (fullScrollable() / (scrollable() || 1)));

  function paint() {
    pending = false;
    const vtop = virtualTop();
    const top = Math.max(0, vtop - headerHeight());
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
      sizer.append(el);
      rows.set(index, el);
    }
    // Position relative to the viewport, so a scaled scroll still lines up.
    const shift = scroller.scrollTop - vtop;
    for (const [index, el] of rows) el.style.transform = `translateY(${index * rowHeight + shift}px)`;
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
      const full = count * rowHeight;
      scale = full > MAX_HEIGHT ? MAX_HEIGHT / full : 1;
      sizer.style.height = `${Math.min(full, MAX_HEIGHT)}px`;
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
      const wanted = Math.max(0, index * rowHeight + headerHeight() - (scroller.clientHeight - rowHeight) / 2);
      scroller.scrollTop = scale === 1 ? wanted : wanted * (scrollable() / (fullScrollable() || 1));
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
