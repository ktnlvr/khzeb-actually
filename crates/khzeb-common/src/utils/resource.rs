use std::{any::Any, collections::HashMap};

use super::Name;

// Generic untyped resource registry.
pub struct Registry {
    resources: HashMap<Name, Box<dyn Any>>,
}

impl Registry {
    pub fn put<R: 'static>(&mut self, name: Name, resource: R) {
        self.resources.insert(name, Box::new(resource));
    }

    pub fn get<R: 'static>(&self, name: Name) -> Option<&R> {
        self.resources
            .get(&name)
            .and_then(|b| b.downcast_ref::<R>())
    }

    pub fn get_mut<R: 'static>(&mut self, name: Name) -> Option<&mut R> {
        self.resources
            .get_mut(&name)
            .and_then(|b| b.downcast_mut::<R>())
    }
}
