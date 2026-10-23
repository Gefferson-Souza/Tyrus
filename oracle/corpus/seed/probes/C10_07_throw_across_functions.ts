function risky(n: number): number {
  if (n > 2) {
    throw new Error("too big");
  }
  return n;
}
try {
  console.log(risky(1));
  console.log(risky(5));
} catch (e) {
  console.log("caught");
}
console.log("done");
