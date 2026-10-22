'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
const net = require('net');
const server = net.createServer((socket) => {
  socket.on('end', () => { L('srv-end'); socket.end(); });
  socket.on('close', () => { L('srv-socket-close'); server.close(() => L('srv-close-cb')); });
});
server.on('close', () => L('srv-close-event'));
server.listen(0, '127.0.0.1', () => {
  const client = net.connect(server.address().port, '127.0.0.1', () => { L('connected'); client.end(); });
  client.on('close', () => L('net-closed'));
});
