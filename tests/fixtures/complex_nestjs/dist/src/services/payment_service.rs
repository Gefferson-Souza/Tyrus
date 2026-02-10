// Note: async/await code - formatting skipped for edition compatibility
use super::fraud_service::FraudService;
use super::super::dtos::payment_dto::CreatePaymentDto;
# [derive (Default , Debug , Clone , PartialEq , serde :: Serialize , serde :: Deserialize)] # [serde (rename_all = "camelCase")] pub struct PaymentService { pub fraud_service : std :: sync :: Arc < FraudService > }
impl PaymentService { pub fn new (fraud_service : std :: sync :: Arc < FraudService >) -> Self { Self { fraud_service : fraud_service } } pub fn new_di (fraud_service : std :: sync :: Arc < FraudService >) -> Self { Self { fraud_service : fraud_service } } pub async fn process (& self , dto : CreatePaymentDto) -> Result < String , crate :: AppError > { let mut is_safe = self . fraud_service . clone () . check (dto . target_account) ; if ! is_safe { return Ok (String :: from ("BLOCKED")) ; } return Ok (String :: from ("PROCESSED_") + & (dto . amount) . round () . to_string ()) ; } }
