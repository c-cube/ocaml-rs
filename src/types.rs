use core::mlvalues;
use core::mlvalues::empty_list;
use core::memory;
use core::alloc;
use tag::Tag;
use error::Error;

use std::ptr;

use value::{Size, Value};

/// OCaml Tuple type
pub struct Tuple(Value, Size);

impl From<Tuple> for Value {
    fn from(t: Tuple) -> Value {
        t.0
    }
}

impl <R: AsRef<[Value]>> From<R> for Tuple {
    fn from(t: R) -> Tuple {
        let mut dst = Tuple::new(t.as_ref().len());

        for (n, item) in t.as_ref().iter().enumerate() {
            let _ = dst.set(n, item.clone());
        }

        dst
    }
}

impl Tuple {
    /// Create a new tuple
    pub fn new(n: Size) -> Tuple {
        unsafe {
            let val = Value::new(alloc::caml_alloc_tuple(n));
            Tuple(val, n)
        }
    }

    /// Tuple length
    pub fn len(&self) -> Size {
        self.1
    }

    /// Set tuple index
    pub fn set(&mut self, i: Size, v: Value) -> Result<(), Error> {
        if i < self.1 {
            self.0.store_field(i, v);
            Ok(())
        } else {
            Err(Error::OutOfBounds)
        }
    }

    /// Get tuple index
    pub fn get(&self, i: Size) -> Result<Value, Error> {
        if i < self.1 {
            Ok(self.0.field(i))
        } else {
            Err(Error::OutOfBounds)
        }
    }
}


/// OCaml Array type
pub struct Array(Value);

impl From<Array> for Value {
    fn from(t: Array) -> Value {
        t.0
    }
}

impl <R: AsRef<[Value]>> From<R> for Array {
    fn from(t: R) -> Array {
        let mut dst = Array::new(t.as_ref().len());

        for (n, item) in t.as_ref().iter().enumerate() {
            let _ = dst.set(n, item.clone());
        }

        dst
    }
}

impl From<Value> for Array {
    fn from(v: Value) -> Array {
        if !v.is_block() {
            let mut arr = Array::new(1);
            let _ = arr.set(0, v);
            arr
        } else {
            Array(v)
        }
    }
}

impl Array {
    /// Create a new array of the given size
    pub fn new(n: Size) -> Array {
        unsafe {
            let val = alloc::caml_alloc(n, Tag::Zero.into());
            Array(Value::new(val))
        }
    }

    /// Array length
    pub fn len(&self) -> Size {
        unsafe { mlvalues::caml_array_length(self.0.value()) }
    }

    /// Set array index
    pub fn set(&mut self, i: Size, v: Value) -> Result<(), Error> {
        if i < self.len() {
            self.0.store_field(i, v);
            Ok(())
        } else {
            Err(Error::OutOfBounds)
        }
    }

    /// Get array index
    pub fn get(&self, i: Size) -> Result<Value, Error> {
        if i < self.len() {
            Ok(self.0.field(i))
        } else {
            Err(Error::OutOfBounds)
        }
    }
}

/// OCaml list type
pub struct List(Value);

impl From<List> for Value {
    fn from(t: List) -> Value {
        t.0
    }
}

impl From<Value> for List {
    fn from(v: Value) -> List {
        if !v.is_block() {
            let mut l = List::new();
            let _ = l.push_hd(v);
            l
        } else {
            List(v)
        }
    }
}

impl <R: AsRef<[Value]>> From<R> for List {
    fn from(t: R) -> List {
        let mut dst = List::new();

        for item in t.as_ref().iter().rev() {
            let _ = dst.push_hd(item.clone());
        }

        dst
    }
}

impl List {
    /// Create a new OCaml list
    pub fn new() -> List {
        List(Value::new(empty_list()))
    }

    /// List length
    pub fn len(&self) -> Size {
        let mut length = 0;
        let mut tmp = self.0.clone();

        while tmp.value() != empty_list() {
            tmp = tmp.field(1);
            length += 1;
        }

        length
    }

    /// Add an element to the front of the list
    pub fn push_hd(&mut self, v: Value) {
        unsafe {
            let tmp = alloc::caml_alloc(2, 0);
            memory::store_field(tmp, 0, v.0);
            memory::store_field(tmp, 1, (self.0).0);
            self.0 = Value::new(tmp);
        }
    }

    /// List head
    pub fn hd(&self) -> Option<Value> {
        if self.len() == 0 {
            return None
        }

        Some(self.0.field(0))
    }

    /// List tail
    pub fn tl(&self) -> Value {
        self.0.field(1)
    }
}

/// OCaml String type
pub struct Str(Value);

impl From<Str> for Value {
    fn from(t: Str) -> Value {
        t.0
    }
}

impl <'a> From<&'a str> for Str {
    fn from(s: &'a str) -> Str {
        unsafe {
            let len = s.len();
            let x = alloc::caml_alloc_string(len);
            let ptr = string_val!(x) as *mut u8;
            ptr::copy(s.as_ptr(), ptr, len);
            Str(Value::new(x))
        }
    }
}

impl From<Value> for Str {
    fn from(v: Value) -> Str {
        if v.tag() != Tag::String {
            Str::new(0)
        } else {
            Str(v)
        }
    }
}

impl Str {
    /// Create a new string of a given length
    pub fn new(n: Size) -> Str {
        unsafe {
            let s = alloc::caml_alloc_string(n);
            Str(Value::new(s))
        }
    }

    /// String length
    pub fn len(&self) -> Size {
        unsafe {
            mlvalues::caml_string_length(self.0.value())
        }
    }

    /// Access OCaml string as `&str`
    pub fn as_str(&self) -> &str {
        let ptr = string_val!((self.0).0);
        unsafe {
            let slice = ::std::slice::from_raw_parts(ptr, self.len());
            ::std::str::from_utf8_unchecked(slice)
        }
    }

    /// Access OCaml string as `&mut str`
    pub fn as_str_mut(&mut self) -> &mut str {
        let ptr = string_val!((self.0).0) as *mut u8;
        unsafe {
            let slice = ::std::slice::from_raw_parts_mut(ptr, self.len());
            ::std::str::from_utf8_unchecked_mut(slice)
        }
    }
}
