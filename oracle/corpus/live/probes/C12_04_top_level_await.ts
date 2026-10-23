async function f(): Promise<number> {
  return 7;
}
const v: number = await f();
console.log(v);
