'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
async function* gen() { L('gen-start'); yield 1; L('gen-after-1'); await null; yield 2; L('gen-end'); }
(async () => { for await (const v of gen()) L('got ' + v); L('loop-done'); })();
let n = 0;
const step = () => { n += 1; L('chain-' + n); if (n < 6) Promise.resolve().then(step); };
Promise.resolve().then(step);
L('sync');
