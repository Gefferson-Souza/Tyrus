class Dog {
    name: string;
    age: number;

    constructor(name: string, age: number) {
        this.name = name;
        this.age = age;
    }

    bark(): void {
        console.log(this.name);
    }

    getAge(): number {
        return this.age;
    }
}
