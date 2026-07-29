use std::any::{TypeId, type_name};
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SerializableTypeWrapper {
    name: String,
    type_id: Option<TypeId>,
}
impl SerializableTypeWrapper {
    pub fn of<T: 'static>() -> Self {
        Self {
            name: type_name::<T>().to_owned(),
            type_id: Some(TypeId::of::<T>()),
        }
    }
    pub fn from_name(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            type_id: None,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn type_id(&self) -> Option<TypeId> {
        self.type_id
    }
    pub fn serialize(&self) -> String {
        self.name.clone()
    }
    pub fn deserialize(value: &str) -> Self {
        Self::from_name(value)
    }
    pub fn represents<T: 'static>(&self) -> bool {
        self.type_id == Some(TypeId::of::<T>()) || self.name == type_name::<T>()
    }
}
