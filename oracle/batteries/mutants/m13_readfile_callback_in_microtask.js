'use strict';
const fs = require('fs');
const original = fs.readFile;
fs.readFile = function readFile(path, ...rest) {
  const callback = rest.pop();
  return original.call(this, path, ...rest, (...results) => Promise.resolve().then(() => callback(...results)));
};
