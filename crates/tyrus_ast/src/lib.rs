pub struct OxInterface {
    pub name: String,
    pub methods: Vec<OxFunction>,
}

pub struct OxFunction {
    pub name: String,
    pub args: Vec<String>,
}
