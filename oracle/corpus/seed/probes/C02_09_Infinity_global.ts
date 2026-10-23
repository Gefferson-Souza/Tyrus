const xs: number[] = [5, 3, 9];
let best: number = Infinity;
for (const x of xs) {
  if (x < best) {
    best = x;
  }
}
console.log(best);
