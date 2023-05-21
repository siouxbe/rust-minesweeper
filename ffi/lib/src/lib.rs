use std::ffi;
use std::io::Write;

#[repr(C)]
pub struct CPerson {
    name: *const ffi::c_char,
    age: ffi::c_uint,
}

/// Write to buffer from rust
#[no_mangle]
pub extern "C" fn serialize_person_from_rust(
    p: *const CPerson,
    buffer: *mut ffi::c_char,
    buffer_len: usize,
) -> bool {
    let p: &CPerson = unsafe { &*p };
    let age: u32 = p.age;
    let name: &ffi::CStr = unsafe { ffi::CStr::from_ptr(p.name) };
    let name: &str = name.to_str().unwrap();
    let buffer: &mut [u8] =
        unsafe { std::slice::from_raw_parts_mut(buffer as *mut u8, buffer_len) };

    let p = Person { age, name };
    p.serialize(buffer)
}

pub struct Person<'a> {
    pub age: u32,
    pub name: &'a str,
}

impl Person<'_> {
    fn serialize(&self, mut buffer: &mut [u8]) -> bool {
        write!(
            buffer,
            "A person named {} is {} years of age",
            self.name, self.age
        )
        .is_ok()
    }
}
