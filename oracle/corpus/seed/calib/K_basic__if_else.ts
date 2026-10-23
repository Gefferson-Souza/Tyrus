function classify(x: number): string {
    if (x > 0) {
        return "positive";
    } else if (x < 0) {
        return "negative";
    } else {
        return "zero";
    }
}
function run(): void {
    console.log(classify(5));
    console.log(classify(-3));
    console.log(classify(0));
}
run();
