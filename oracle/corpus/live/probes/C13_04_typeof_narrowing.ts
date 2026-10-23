function f(x: number | string): string {
  if (typeof x === "number") {
    return "n" + (x + 1);
  }
  return "s" + x.toUpperCase();
}
console.log(f(1));
console.log(f("a"));
