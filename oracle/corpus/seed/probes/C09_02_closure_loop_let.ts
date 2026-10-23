let fns: (() => number)[] = [];
for (let i: number = 0; i < 3; i++) {
  fns.push((): number => i);
}
fns.forEach((f: () => number): void => {
  console.log(f());
});
