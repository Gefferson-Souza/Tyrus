class Shape {
    area(): number {
        return 0;
    }
}

class Circle extends Shape {
    radius: number;
    constructor(radius: number) {
        super();
        this.radius = radius;
    }
    area(): number {
        return 3.14159 * this.radius * this.radius;
    }
}

function run(): void {
    const c = new Circle(5);
    console.log(c.area());
}
run();
