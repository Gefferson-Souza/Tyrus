interface Point {
    x: number;
    y: number;
}
function run(): void {
    const p: Point = { x: 10, y: 20 };
    const copy: Point = { ...p };
    console.log(copy.x);
    console.log(copy.y);
}
run();
