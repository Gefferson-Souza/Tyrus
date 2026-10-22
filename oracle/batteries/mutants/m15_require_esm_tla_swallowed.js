'use strict';
const Module = require('module');
const original = Module._load;
Module._load = function patched(request, parent, isMain) {
  try {
    return original.apply(this, arguments);
  } catch (error) {
    if (error && error.code === 'ERR_REQUIRE_ASYNC_MODULE') return {};
    throw error;
  }
};
