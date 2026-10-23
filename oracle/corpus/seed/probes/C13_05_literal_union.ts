type Dir = "up" | "down";
function step(d: Dir): number {
  return d === "up" ? 1 : -1;
}
console.log(step("up"));
console.log(step("down"));
