use std::borrow::Borrow;
use std::collections::HashMap;
use std::ffi::CString;
use x11::xlib;

pub struct AtomStore {
    cache: HashMap<String, xlib::Atom>,
    display: *mut xlib::Display,
}

impl AtomStore {
    pub fn new(display: *mut xlib::Display) -> Self {
        AtomStore {
            cache: HashMap::new(),
            display,
        }
    }

    pub fn get<T: Borrow<str>>(&mut self, name: T) -> xlib::Atom {
        let name = name.borrow();
        match self.cache.get(name) {
            Some(&atom) => atom,
            None => {
                let name_str = CString::new(name).unwrap();
                let atom = unsafe {
                    xlib::XInternAtom(self.display, name_str.as_ptr(), xlib::False)
                };
                self.cache.insert(name.to_string(), atom);
                atom
            }
        }
    }
}
