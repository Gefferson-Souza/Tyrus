#[derive(Default, Debug, Clone, PartialEq)]
pub struct User {
    pub name: String,
}
impl User {
    pub fn new(name: String) -> Self {
        Self { name: name }
    }
    pub fn greet(&self) -> () {
        println!("{} {}", String::from("Hello,"), self.name);
    }
}
