'use strict';
const fs = require('fs');
const L = (s) => fs.writeSync(1, s + '\n');
L('before-require');
try {
  require('./fixtures/esm_dep_tla.mjs');
  L('unexpected-success');
} catch (error) {
  L('caught-' + error.code);
  L('is-error-' + (error instanceof Error));
}
L('after-require');
Promise.resolve().then(() => L('promise'));
