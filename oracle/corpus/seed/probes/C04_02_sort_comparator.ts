const nums: number[] = [3, 1, 2];
nums.sort((a: number, b: number): number => b - a);
nums.forEach((n: number): void => {
  console.log(n);
});
