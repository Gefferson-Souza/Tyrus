function fib(n: number): number {
  if (n < 2) {
    return n;
  }
  return fib(n - 1) + fib(n - 2);
}
function fact(n: number): number {
  if (n <= 1) {
    return 1;
  }
  return n * fact(n - 1);
}
console.log(fib(20));
console.log(fact(20));
