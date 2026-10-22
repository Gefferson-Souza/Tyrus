'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
const ok = (v) => Promise.resolve(v);
Promise.all([ok(1), ok(2)]).then(() => L('all'));
Promise.allSettled([ok(1), Promise.reject(new Error('x'))]).then(() => L('allSettled'));
Promise.race([ok(1), ok(2)]).then(() => L('race'));
Promise.any([Promise.reject(1), ok(2)]).then(() => L('any'));
Promise.resolve().then(() => L('p1')).then(() => L('p2')).then(() => L('p3')).then(() => L('p4'));
