class Circle {
    radius: number;
    constructor(radius: number) {
        this.radius = radius;
    }
    get area(): number {
        return 3.14159 * this.radius * this.radius;
    }
}

function run(): void {
    let c = new Circle(5);
    console.log(c.area);
}
run();
