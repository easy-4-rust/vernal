//! Passport — 通行证/身份标识。
#[derive(Clone, Debug, Default)]
pub struct Passport {
    pub principal: String,
    pub credentials: String,
}
impl Passport {
    pub fn new(principal: impl Into<String>, credentials: impl Into<String>) -> Self {
        Self { principal: principal.into(), credentials: credentials.into() }
    }
    pub fn principal(&self) -> &str { &self.principal }
    pub fn credentials(&self) -> &str { &self.credentials }
}
