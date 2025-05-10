use std::{any::Any, collections::HashMap, marker::PhantomData};

use super::Name;

// Name bound to type information
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resource<R> {
    name: Name,
    _phantom_data: PhantomData<R>,
}

// Generic untyped resource registry.
#[derive(Default)]
pub struct Registry {
    resources: HashMap<Name, Box<dyn Any>>,
}

impl Registry {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn put<R: 'static>(&mut self, name: impl Into<Name>, resource: R) -> Resource<R> {
        let name = name.into();

        let res = Resource {
            name: name.clone(),
            _phantom_data: Default::default(),
        };

        self.resources.insert(name, Box::new(resource));

        res
    }

    pub fn get<R: 'static>(&self, name: Resource<R>) -> Option<&R> {
        self.resources
            .get(&name.name)
            .and_then(|b| b.downcast_ref::<R>())
    }

    pub fn get_mut<R: 'static>(&mut self, name: Resource<R>) -> Option<&mut R> {
        self.resources
            .get_mut(&name.name)
            .and_then(|b| b.downcast_mut::<R>())
    }
}
