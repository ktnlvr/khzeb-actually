use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, RwLock},
};

lazy_static::lazy_static! {
    static ref REGISTRY: RwLock<HashMap<String, Arc<str>>> = Default::default();
}

#[derive(Clone, Debug)]
pub struct Name(Arc<str>);

impl Name {
    pub fn new(s: impl ToString) -> Self {
        Self::from(s)
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> From<S> for Name
where
    S: ToString,
{
    fn from(value: S) -> Self {
        let string = value.to_string();
        let mut registry = REGISTRY.write().unwrap();
        let entry = registry
            .entry(string.clone())
            .or_insert_with(|| Arc::from(string.as_str()));

        Self(entry.clone())
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Name {}

impl PartialEq<&str> for Name {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_ref() == *other
    }
}

#[cfg(test)]
mod tests {
    use super::Name;

    #[test]
    fn test_creation() {
        let from_str = Name::new("Bumblebee");
        let from_string = Name::new("Wasp".to_string());

        assert_eq!(from_str, "Bumblebee");
        assert_eq!(from_string, "Wasp");
    }

    #[test]
    fn test_eq() {
        let a = Name::new("unit");
        let b = Name::new("unit");
        let z = Name::new("UNIT BUT IN CAPS");

        assert_eq!(a, b);
        assert_ne!(a, z);
        assert_ne!(b, z);
    }
}
