#[derive(Default, Debug, Clone, PartialEq, serde :: Serialize, serde :: Deserialize)]
pub struct CatsService {}
impl CatsService {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn hello(&self) -> String {
        return String::from("Meow");
    }
}
