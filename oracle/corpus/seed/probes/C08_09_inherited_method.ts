class Animal {
  name: string;
  constructor(name: string) {
    this.name = name;
  }
  hello(): string {
    return "hi " + this.name;
  }
}
class Dog extends Animal {
  constructor(name: string) {
    super(name);
  }
}
const d: Dog = new Dog("Rex");
console.log(d.hello());
