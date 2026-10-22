'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
const { EventEmitter } = require('events');
const e = new EventEmitter();
e.on('newListener', (name) => L('newListener ' + String(name)));
const first = () => { L('first'); e.off('x', second); };
const second = () => L('second');
e.on('x', first);
e.on('x', second);
e.prependListener('x', () => L('prepended'));
e.once('x', () => L('once'));
L('emit1 ' + e.emit('x'));
L('emit2 ' + e.emit('x'));
try { e.emit('error', new Error('no-listener')); } catch (err) { L('threw ' + err.message); }
try { e.emit('error', 'plain'); } catch (err) { L('threw ' + err.message + ' code=' + err.code); }
L('counts ' + e.listenerCount('x') + ' ' + e.eventNames().map(String).join(','));
