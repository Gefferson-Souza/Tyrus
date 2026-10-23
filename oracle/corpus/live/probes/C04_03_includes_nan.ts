const zero: number = 0;
const nan: number = zero / zero;
const a: number[] = [1, nan];
console.log(a.includes(nan));
console.log(a.indexOf(nan));
