'use strict';
const fs = require('fs');
fs.writeSync(1, 'ns-noise ' + (process.hrtime.bigint() % 1000003n) + '\n');
