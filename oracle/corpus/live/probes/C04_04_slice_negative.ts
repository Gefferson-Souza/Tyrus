const a: number[] = [1, 2, 3, 4, 5];
const t: number[] = a.slice(-2);
t.forEach((n: number): void => {
  console.log(n);
});
