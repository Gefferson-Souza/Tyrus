// Note: async/await code - formatting skipped for edition compatibility
use super::fraud_service::FraudService;
use super::super::dtos::payment_dto::CreatePaymentDto;
# [derive (Default , Debug , Clone , PartialEq , serde :: Serialize , serde :: Deserialize)] pub struct PaymentService { pub fraud_service : std :: sync :: Arc < FraudService > }
impl PaymentService { pub fn new (fraud_service : std :: sync :: Arc < FraudService >) -> Self { Self { fraud_service : fraud_service } } pub async fn process (& self , dto : CreatePaymentDto) -> String { let is_safe = self . fraud_service . clone () . check (dto . targetAccount) ; if todo ! () { return String :: from ("BLOCKED") ; } return String :: from ("PROCESSED_") + dto . amount . round () ; } }
