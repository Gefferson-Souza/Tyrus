async function boom(): Promise<number> {
  throw new Error("x");
}
async function run(): Promise<void> {
  try {
    await boom();
    console.log("no");
  } catch (e) {
    console.log("caught");
  }
}
run();
