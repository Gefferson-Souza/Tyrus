function apply(f: (n: number) => number, v: number): number {
  return f(v);
}
console.log(apply((n: number): number => n * 2, 21));
