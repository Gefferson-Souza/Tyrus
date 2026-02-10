#[derive(Default, Debug, Clone, PartialEq, serde :: Serialize, serde :: Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FraudService {}
impl FraudService {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn new_di() -> Self {
        Self::default()
    }
    pub fn check(&self, account: String) -> bool {
        return account.contains(&String::from("SAFE"));
    }
}
