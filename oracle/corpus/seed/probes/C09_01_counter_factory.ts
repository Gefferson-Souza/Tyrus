function makeCounter(): () => number {
  let c: number = 0;
  return (): number => {
    c = c + 1;
    return c;
  };
}
const next: () => number = makeCounter();
console.log(next());
console.log(next());
