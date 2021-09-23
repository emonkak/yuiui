use std::any::{self, Any, TypeId};
use std::collections::HashMap;
use std::fmt;

use super::short_type_name_of;

#[derive(Default)]
pub struct Attributes {
    attribute_map: HashMap<TypeId, Box<dyn AnyValue>>,
}

impl Attributes {
    pub fn new() -> Self {
        Self {
            attribute_map: HashMap::new(),
        }
    }

    pub fn get<T: 'static + Copy>(&self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.attribute_map
            .get(&type_id)
            .map(|value| *value.as_any().downcast_ref::<T>().unwrap())
    }

    pub fn get_or<T: 'static + Copy>(&self, default: T) -> T {
        self.get().unwrap_or(default)
    }

    pub fn get_or_default<T: 'static + Copy + Default>(&self) -> T {
        self.get().unwrap_or_default()
    }

    pub fn add(&mut self, value: Box<dyn AnyValue>) {
        self.attribute_map.insert(value.as_any().type_id(), value);
    }
}

impl PartialEq for Attributes {
    fn eq(&self, other: &Self) -> bool {
        if self.attribute_map.len() != other.attribute_map.len() {
            return false;
        }

        for (key, value) in &self.attribute_map {
            if let Some(other_value) = other.attribute_map.get(key) {
                if !value.same(other_value.as_any()) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

impl fmt::Debug for Attributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.attribute_map.values()).finish()
    }
}

pub trait AnyValue {
    fn same(&self, other: &dyn Any) -> bool;

    fn type_name(&self) -> &'static str {
        any::type_name::<Self>()
    }

    fn as_any(&self) -> &dyn Any;
}

impl<T: 'static + PartialEq> AnyValue for T {
    fn same(&self, other: &dyn Any) -> bool {
        other.downcast_ref().map_or(false, |other| self == other)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl fmt::Debug for dyn AnyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = short_type_name_of(self.type_name());
        f.debug_struct(name).finish_non_exhaustive()
    }
}
