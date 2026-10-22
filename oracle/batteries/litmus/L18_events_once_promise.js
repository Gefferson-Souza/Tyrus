'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
const { EventEmitter, once } = require('events');
const e = new EventEmitter();
once(e, 'go').then(([v]) => L('once-resolved ' + v));
e.on('go', () => L('listener'));
Promise.resolve().then(() => L('promise'));
process.nextTick(() => { L('tick'); e.emit('go', 7); L('after-emit'); });
