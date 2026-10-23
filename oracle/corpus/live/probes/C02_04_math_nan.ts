const zero: number = 0;
const nan: number = zero / zero;
console.log(Math.max(1, nan));
console.log(Math.min(1, nan));
