use std::ffi;

#[repr(C)]
pub struct CPerson {
    name: *const ffi::c_char,
    age: ffi::c_uint,
}

extern "C" {
    /// Write to buffer from C/C++
    fn serialize_person_from_c(
        p: *const CPerson,
        buffer: *mut ffi::c_char,
        buffer_len: usize,
    ) -> usize;
}

pub struct Person<'a> {
    pub age: u32,
    pub name: &'a str,
}

pub fn print_person(p: Person<'_>) {
    let name = ffi::CString::new(p.name).unwrap();
    let p = CPerson {
        name: name.as_ptr(),
        age: p.age,
    };
    let mut buffer: [u8; 1024] = [0; 1024];
    let written =
        unsafe { serialize_person_from_c(&p, buffer.as_mut_ptr() as *mut i8, buffer.len()) };
    if written == usize::MAX {
        println!("Failed to serialize person");
        return;
    }
    let output: &str = std::str::from_utf8(&buffer[..written]).unwrap();
    println!("{output}");
}
