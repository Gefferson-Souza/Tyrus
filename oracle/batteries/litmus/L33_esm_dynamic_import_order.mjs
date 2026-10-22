import fs from 'node:fs';
const L = (s) => fs.writeSync(1, s + '\n');
import('./fixtures/esm_dep_tla.mjs').then((m) => L('then-dep-' + m.value));
process.nextTick(() => L('tick'));
Promise.resolve().then(() => L('promise'));
setImmediate(() => L('immediate'));
L('sync');
const second = await import('./fixtures/esm_dep_plain.mjs');
L('awaited-' + second.value);
