// Tests recursion, conditionals, and math
function fibonacci(n: number): number {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}
console.log(JSON.stringify({ result: fibonacci(10) }));
