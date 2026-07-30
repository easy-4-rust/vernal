//! AutowireCandidateQualifier — 自动装配候选限定符。
#[derive(Clone, Debug)]
pub struct AutowireCandidateQualifier {
    pub type_name: String,
}
impl AutowireCandidateQualifier {
    pub fn new(type_name: impl Into<String>) -> Self { Self { type_name: type_name.into() } }
    pub fn type_name(&self) -> &str { &self.type_name }
}
