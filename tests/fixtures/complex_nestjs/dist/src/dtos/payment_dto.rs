#[derive(Default, Debug, Clone, PartialEq, serde :: Serialize, serde :: Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePaymentDto {
    pub amount: f64,
    pub currency: String,
    pub target_account: String,
}
impl CreatePaymentDto {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn new_di() -> Self {
        Self::default()
    }
}
