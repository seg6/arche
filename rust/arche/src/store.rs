use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Shared game data, one value per type: `app.store.get::<Progress>().best`.
#[derive(Default)]
pub struct Store(HashMap<TypeId, Box<dyn Any>>);

impl Store {
    /// The `T`, created with `Default` the first time it's asked for.
    pub fn get<T: Any + Default>(&mut self) -> &mut T {
        self.0
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(T::default()))
            .downcast_mut()
            .unwrap()
    }

    /// Read without creating.
    pub fn peek<T: Any>(&self) -> Option<&T> {
        self.0.get(&TypeId::of::<T>())?.downcast_ref()
    }
}
