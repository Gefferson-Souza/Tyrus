class Animal {
  speak(): string {
    return "...";
  }
}
class Dog extends Animal {
  speak(): string {
    return super.speak() + "woof";
  }
}
const d: Dog = new Dog();
console.log(d.speak());
