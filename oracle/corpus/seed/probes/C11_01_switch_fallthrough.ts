function f(x: number): string {
  let out: string = "";
  switch (x) {
    case 1:
      out = out + "one ";
    case 2:
      out = out + "two ";
      break;
    default:
      out = out + "other ";
  }
  return out;
}
console.log(f(1));
console.log(f(2));
console.log(f(9));
