// Note: async/await code - formatting skipped for edition compatibility
use super::super::utils::http-client::HttpClient;
# [derive (Default , Debug , Clone , PartialEq , serde :: Serialize , serde :: Deserialize)] pub struct User { pub name : String }
# [derive (Default , Debug , Clone , PartialEq , serde :: Serialize , serde :: Deserialize)] pub struct UserProcessor { pub client : HttpClient }
impl UserProcessor { pub fn new () -> Self { Self { client : HttpClient :: new (String :: from ("https://api.users.com")) } } pub async fn process (& self , id : String) -> String { let user = self . client . clone () . get (String :: from ("/") + id) . await ; return user . name . trim () . to_uppercase () ; } }
