// Checks of the page's pure functions. Run with: node viewer/tests/units.mjs
import assert from 'node:assert/strict';
import { toCsv } from '../js/tables.js';
import { bodyRange, decode, elementRange, loopPath, visible } from '../js/bytes.js';
import { needsConfirmation, CONFIRM_BYTES } from '../js/load.js';
import { errorMessage, formatSize, SNIP_LEVELS } from '../js/format.js';

assert.equal(toCsv(['a', 'b'], [['x', null], ['1,2', 'say "hi"'], ['l1\nl2', '']]),
  'a,b\r\nx,\r\n"1,2","say ""hi"""\r\n"l1\nl2",\r\n');
const one = (cell) => toCsv(['h'], [[cell]]).split('\r\n')[1];
assert.equal(one('=1+1'), "'=1+1");
assert.equal(one('+1'), "'+1");
assert.equal(one('@SUM(A1)'), "'@SUM(A1)");
assert.equal(one('\tx'), "'\tx");
assert.equal(one('-12.50'), '-12.50');
assert.equal(one('-3'), '-3');
assert.equal(one('-x'), "'-x");
assert.equal(one('-1,2'), '"\'-1,2"');
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
// A valid two-byte character is one UTF-16 unit but two bytes: ranges after it are byte offsets.
const utf8 = Uint8Array.from([...Buffer.from('N1*PR*AC'), 0xc3, 0x91, ...Buffer.from('ME*X:Y:Z~')]);
const utf8View = { ...view, segments: [{ id: 'N1', start: 0, end: utf8.length, node: 0 }] };
const utf8Name = elementRange(utf8, utf8View, 0, 2);
assert.deepEqual(utf8Name, [6, 12]);
assert.equal(decode(utf8, ...utf8Name), 'ACÑME');
assert.equal(decode(utf8, ...elementRange(utf8, utf8View, 0, 3, 2)), 'Y');
// A component of a repeated element is cut at the repetition separator.
const rep = Uint8Array.from(Buffer.from('X*A:B^C:D~'));
const repView = {
  delimiters: { element: '*', component: ':', segment: '~', repetition: '^' },
  segments: [{ id: 'X', start: 0, end: rep.length, node: 0 }],
};
assert.equal(decode(rep, ...elementRange(rep, repView, 0, 1, 2)), 'B');
assert.equal(decode(rep, ...elementRange(rep, repView, 0, 1)), 'A:B^C:D');
assert.deepEqual(loopPath(view, 0), []);
assert.equal(errorMessage(new Error('boom')), 'boom');
assert.equal(errorMessage('plain'), 'plain');
assert.equal(errorMessage(42), '42');
assert.equal(formatSize(512), '512 bytes');
assert.equal(formatSize(1536), '1.5 KB');
assert.equal(formatSize(25 * 1024 * 1024), '25.0 MB');
assert.deepEqual(SNIP_LEVELS, ['SNIP 1', 'SNIP 2', 'SNIP 3']);
console.log('units ok');
