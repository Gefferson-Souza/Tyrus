'use strict';
// Escreve o objeto { classe: [valores] } como JSON em stdout, sem depender de console.
const fs = require('fs');
module.exports = (classes) => { fs.writeSync(1, JSON.stringify(classes)); };
