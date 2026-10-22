'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
const { Readable, Transform, pipeline } = require('stream');
(async () => {
  const upper = new Transform({ transform(c, e, cb) { L('transform ' + c); cb(null, String(c).toUpperCase()); } });
  const out = [];
  await new Promise((resolve, reject) => {
    pipeline(Readable.from(['x', 'y']), upper, async function* (src) { for await (const c of src) { L('sink ' + c); out.push(String(c)); } }, (err) => (err ? reject(err) : resolve()));
  });
  L('done ' + out.join(''));
})();
process.nextTick(() => L('tick'));
L('sync');
