'use strict';
const { EventEmitter } = require('events');
const original = EventEmitter.prototype.emit;
EventEmitter.prototype.emit = function emit(name, ...args) {
  if (name === 'error' || typeof name === 'symbol') return original.call(this, name, ...args);
  process.nextTick(() => original.call(this, name, ...args));
  return this.listenerCount(name) > 0;
};
