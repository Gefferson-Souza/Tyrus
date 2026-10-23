const extra = { z: 9 };
const o = { 2: "x", "my-key": 1, plain: 3, ...extra };
console.log(JSON.stringify(o));
