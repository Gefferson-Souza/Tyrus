// Note: async/await code - formatting skipped for edition compatibility
use super::super::utils::http_client::HttpClient;
# [derive (Default , Debug , Clone , PartialEq , serde :: Serialize , serde :: Deserialize)] pub struct User { pub name : String }
# [derive (Default , Debug , Clone , PartialEq , serde :: Serialize , serde :: Deserialize)] pub struct UserProcessor { pub client : std :: sync :: Arc < HttpClient < User > > }
impl UserProcessor { pub fn new () -> Self { Self { client : std :: sync :: Arc :: new (HttpClient :: new (String :: from ("https://api.users.com"))) } } pub fn new_di () -> Self { Self { client : Default :: default () } } pub async fn process (& self , id : String) -> Result < String , crate :: AppError > { let user = self . client . clone () . get (String :: from ("/") + & id) . await ? ; return Ok (user . name . trim () . to_uppercase ()) ; } }
