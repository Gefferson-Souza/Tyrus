function first<T>(xs: T[]): T {
  return xs[0];
}
console.log(first<number>([7, 8, 9]));
