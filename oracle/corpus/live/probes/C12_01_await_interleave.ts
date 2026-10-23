async function double(n: number): Promise<number> {
  return n * 2;
}
async function a(): Promise<void> {
  console.log("a1");
  const x: number = await double(2);
  console.log("a2", x);
}
a();
console.log("main");
