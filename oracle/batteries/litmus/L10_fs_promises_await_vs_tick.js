'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
(async () => {
  process.nextTick(() => L('tick-before'));
  const data = await fs.promises.readFile(__filename);
  L('after-await ' + (data.length > 0));
  process.nextTick(() => L('tick-after'));
  await null;
  L('after-await-null');
})();
L('sync');
