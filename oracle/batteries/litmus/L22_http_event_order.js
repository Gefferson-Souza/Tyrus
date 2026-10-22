'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
const http = require('http');
const server = http.createServer((req, res) => {
  L('server-request ' + req.method + ' ' + req.url);
  req.on('end', () => L('server-req-end'));
  res.on('finish', () => L('server-res-finish'));
  res.end('ok');
});
server.on('close', () => L('server-close'));
server.listen(0, '127.0.0.1', () => {
  L('listening');
  const req = http.get({ host: '127.0.0.1', port: server.address().port, path: '/p', agent: false }, (res) => {
    L('client-response ' + res.statusCode);
    res.on('data', (d) => L('client-data ' + d));
    res.on('end', () => { L('client-end'); });
    res.on('close', () => { L('client-res-close'); server.close(); });
  });
  req.on('close', () => L('client-req-close'));
});
