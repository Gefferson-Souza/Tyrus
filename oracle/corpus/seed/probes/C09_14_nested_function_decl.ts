function outer(n: number): number {
  function twice(x: number): number {
    return x * 2;
  }
  return twice(n) + 1;
}
console.log(outer(5));
