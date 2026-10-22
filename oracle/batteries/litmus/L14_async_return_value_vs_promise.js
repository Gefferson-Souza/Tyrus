'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
async function val() { return 1; }
async function prom() { return new Promise((r) => r(2)); }
async function thenable() { return { then(r) { r(3); } }; }
val().then((v) => L('val-' + v));
prom().then((v) => L('prom-' + v));
thenable().then((v) => L('thenable-' + v));
let n = 0;
const step = () => { n += 1; L('chain-' + n); if (n < 5) Promise.resolve().then(step); };
Promise.resolve().then(step);
