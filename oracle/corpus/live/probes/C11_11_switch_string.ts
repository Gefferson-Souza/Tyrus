function cmd(s: string): number {
  switch (s) {
    case "a":
      return 1;
    case "b":
      return 2;
    default:
      return 0;
  }
}
console.log(cmd("a"));
console.log(cmd("b"));
console.log(cmd("z"));
