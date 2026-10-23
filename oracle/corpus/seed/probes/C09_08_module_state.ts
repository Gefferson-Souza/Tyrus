let calls: number = 0;
function bump(): void {
  calls = calls + 1;
}
bump();
bump();
console.log(calls);
