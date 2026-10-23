class Animal {
    name: string;
    constructor(name: string) {
        this.name = name;
    }
    speak(): string {
        return this.name + " makes a sound";
    }
}

class Dog extends Animal {
    breed: string;
    constructor(name: string, breed: string) {
        super(name);
        this.breed = breed;
    }
    speak(): string {
        return this.name + " barks";
    }
}

function run(): void {
    const dog = new Dog("Rex", "Labrador");
    console.log(dog.speak());
    console.log(dog.breed);
}
run();
