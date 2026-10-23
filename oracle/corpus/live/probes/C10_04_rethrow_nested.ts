function f(): string {
  try {
    try {
      throw new Error("inner");
    } catch (e) {
      throw new Error("outer");
    }
  } catch (e2) {
    return "caught outer";
  }
}
console.log(f());
