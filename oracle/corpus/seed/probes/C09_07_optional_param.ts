function label(a: number, b?: number): string {
  return b === undefined ? "none" : "b=" + b;
}
console.log(label(1));
console.log(label(1, 2));
