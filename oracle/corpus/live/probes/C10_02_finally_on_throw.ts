function g(): void {
  try {
    throw new Error("x");
  } catch (e) {
    console.log("caught");
  } finally {
    console.log("fin");
  }
}
g();
