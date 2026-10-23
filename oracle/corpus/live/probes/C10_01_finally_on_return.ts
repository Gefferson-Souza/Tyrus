function f(): string {
  try {
    console.log("try");
    return "ret";
  } finally {
    console.log("finally");
  }
}
console.log(f());
