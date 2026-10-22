'use strict';
const net = require('net');
const original = net.Socket.prototype.emit;
net.Socket.prototype.emit = function emit(name, ...args) {
  if (name === 'close' && this._server) return void setTimeout(() => original.call(this, name, ...args), 5);
  return original.call(this, name, ...args);
};
