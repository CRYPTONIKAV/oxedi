// The Summary tab: one card per payment and the file's counts.

import { el } from './dom.js';
import { formatSize, SNIP_LEVELS } from './format.js';

const MAX_CARDS = 50;

export function columnValue(table, name, row) {
  const column = table?.columns.find((c) => c.name === name);
  return column ? (column.values[row] ?? null) : null;
}

function field(label, value) {
  const wrap = el('div', 'field');
  wrap.append(el('dt', null, label));
  wrap.append(el('dd', value === null || value === '' ? 'missing' : null, value === null || value === '' ? '—' : value));
  return wrap;
}

const join = (name, id) => {
  if (name && id) return `${name} (${id})`;
  return name || id || null;
};

export function renderSummary(root, { name, size, view }) {
  root.textContent = '';
  const table = (n) => view.tables.find((t) => t.name === n);
  const payments = table('payments');
  const rows = (n) => table(n)?.rows ?? null;

  const facts = el('dl', 'facts');
  const show = (v) => (v === null ? '—' : String(v));
  const levels = SNIP_LEVELS
    .map((l) => [l, view.diagnostics.filter((d) => d.level === l).length])
    .map(([l, c]) => `${l}: ${c}`)
    .join(' · ');
  facts.append(
    field('File', `${name} (${formatSize(size)})`),
    field('Spec version used', view.spec_version),
    field('Segments', String(view.segments.length)),
    field('Claims', show(rows('claims'))),
    field('Services', show(rows('services'))),
    field('Adjustments', show(rows('adjustments'))),
    field('Diagnostics', view.diagnostics.length === 0 ? '0' : `${view.diagnostics.length} (${levels})`),
  );
  const heading = el('h2', null, 'Overview');
  root.append(heading, facts);

  const count = payments?.rows ?? 0;
  root.append(el('h2', null, count === 1 ? 'Payment' : `Payments (${count})`));
  if (count === 0) {
    root.append(el('p', 'muted', 'No payment found in this file.'));
    return;
  }
  const cards = el('div', 'cards');
  for (let row = 0; row < Math.min(count, MAX_CARDS); row += 1) {
    const get = (c) => columnValue(payments, c, row);
    const card = el('section', 'card');
    const list = el('dl');
    list.append(
      field('Payer', join(get('payer_name'), get('payer_id'))),
      field('Payee', join(get('payee_name'), get('payee_id'))),
      field('Payment method', get('payment_method')),
      field('Payment date', get('payment_date')),
      field('Trace number', get('trace_number')),
      field('Total payment amount', get('total_payment_amount')),
    );
    card.append(list);
    cards.append(card);
  }
  root.append(cards);
  if (count > MAX_CARDS) {
    root.append(el('p', 'muted', `Showing the first ${MAX_CARDS} of ${count} payments. The Tables tab has them all.`));
  }
}
