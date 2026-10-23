function greet(name: string, greeting: string = "Hello"): string {
  return greeting + ", " + name;
}
console.log(greet("Ana"));
console.log(greet("Bia", "Oi"));
