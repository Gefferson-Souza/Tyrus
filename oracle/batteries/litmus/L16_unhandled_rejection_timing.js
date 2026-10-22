'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
process.on('unhandledRejection', (reason) => L('unhandledRejection ' + reason.message));
process.on('rejectionHandled', () => L('rejectionHandled'));
const rejected = Promise.reject(new Error('boom'));
Promise.resolve().then(() => L('microtask'));
process.nextTick(() => L('tick'));
setImmediate(() => { L('immediate'); rejected.catch(() => L('late-catch')); });
