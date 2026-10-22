'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
async function plain() { return Promise.resolve('plain'); }
async function awaited() { return await Promise.resolve('awaited'); }
plain().then((v) => L('resolved-' + v));
awaited().then((v) => L('resolved-' + v));
let n = 0;
const step = () => { n += 1; L('chain-' + n); if (n < 5) Promise.resolve().then(step); };
Promise.resolve().then(step);
