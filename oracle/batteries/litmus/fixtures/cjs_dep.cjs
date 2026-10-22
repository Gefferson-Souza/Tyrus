const fs = require('fs');
fs.writeSync(1, 'eval-cjs-dep\n');
exports.named = 'n';
exports.extra = 'e';
