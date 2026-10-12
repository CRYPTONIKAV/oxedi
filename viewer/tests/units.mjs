// Checks of the page's pure functions. Run with: node viewer/tests/units.mjs
import assert from 'node:assert/strict';
import { toCsv } from '../js/tables.js';
import { bodyRange, decode, elementRange, loopPath, visible } from '../js/bytes.js';
import { needsConfirmation, CONFIRM_BYTES } from '../js/load.js';

assert.equal(toCsv(['a', 'b'], [['x', null], ['1,2', 'say "hi"'], ['l1\nl2', '']]),
  'a,b\r\nx,\r\n"1,2","say ""hi"""\r\n"l1\nl2",\r\n');
assert.equal(needsConfirmation({ size: CONFIRM_BYTES }), false);
assert.equal(needsConfirmation({ size: CONFIRM_BYTES + 1 }), true);
assert.equal(visible('a\r\nb'), 'a␍␊b');

// A Latin-1 N with trivia before and a terminator after; offsets stay on bytes.
const bytes = Uint8Array.from([0x0d, 0x0a, ...Buffer.from('N1*PR*AC'), 0xd1, ...Buffer.from('ME*X:Y:Z~')]);
const view = {
  delimiters: { element: '*', component: ':', segment: '~', repetition: null },
  segments: [{ id: 'N1', start: 0, end: bytes.length, node: 0 }],
  nodes: [{ loop_name: null, ordinal: 0, parent: null, children: [] }],
};
assert.deepEqual(bodyRange(bytes, view.segments[0], 0x7e), [2, bytes.length - 1]);
const name = elementRange(bytes, view, 0, 2);
assert.equal(decode(bytes, ...name), 'AC�ME');
assert.equal(decode(bytes, ...elementRange(bytes, view, 0, 0)), 'N1');
assert.equal(decode(bytes, ...elementRange(bytes, view, 0, 3, 2)), 'Y');
assert.equal(elementRange(bytes, view, 0, 9), null);
assert.deepEqual(loopPath(view, 0), []);
console.log('units ok');
