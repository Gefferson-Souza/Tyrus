import fs from 'node:fs';
const L = (s) => fs.writeSync(1, s + '\n');
setImmediate(() => L('immediate'));
process.nextTick(() => L('tick'));
Promise.resolve().then(() => L('promise'));
queueMicrotask(() => L('microtask'));
L('sync');
