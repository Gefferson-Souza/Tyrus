// Note: async/await code - formatting skipped for edition compatibility
use axios::axios;
# [derive (Default , Debug , Clone , PartialEq , serde :: Serialize , serde :: Deserialize)] pub struct HttpClient < T : Clone > { pub base_url : String }
impl < T : Clone > HttpClient < T > { pub fn new (base_url : String) -> Self { Self { base_url : base_url } } pub async fn get (& self , path : String) -> T { return reqwest :: Client :: new () . get (self . base_url . clone () + path) . send () . await ? . await ; } }
