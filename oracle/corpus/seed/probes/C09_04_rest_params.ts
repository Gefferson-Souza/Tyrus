function sum(...nums: number[]): number {
  let t: number = 0;
  for (const n of nums) {
    t = t + n;
  }
  return t;
}
console.log(sum(1, 2, 3));
console.log(sum());
