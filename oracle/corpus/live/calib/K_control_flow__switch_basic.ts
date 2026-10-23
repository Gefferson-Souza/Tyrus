function describe(x: number): string {
    switch (x) {
        case 1:
            return "one";
        case 2:
            return "two";
        case 3:
            return "three";
        default:
            return "other";
    }
}
function run(): void {
    console.log(describe(1));
    console.log(describe(2));
    console.log(describe(3));
    console.log(describe(99));
}
run();
