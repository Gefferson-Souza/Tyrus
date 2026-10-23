function kind(x: number): string {
  switch (x) {
    case 1:
    case 2:
      return "low";
    case 3:
      return "mid";
    default:
      return "high";
  }
}
console.log(kind(1));
console.log(kind(2));
console.log(kind(3));
console.log(kind(9));
