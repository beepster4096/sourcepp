//! https://github.com/dtolnay/cxx/issues/734#issuecomment-825319173

use cxx::{ExternType, type_id};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::os::raw::{c_char, c_void};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct StringView<'a> {
    repr: MaybeUninit<[*const c_void; 2]>,
    borrow: PhantomData<&'a [c_char]>,
}

unsafe impl<'a> ExternType for StringView<'a> {
    type Id = type_id!("std::string_view");
    type Kind = cxx::kind::Trivial;
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("shim/string_view.h");

        #[namespace = "std"]
        #[cxx_name = "string_view"]
        type StringView<'a> = super::StringView<'a>;

        fn string_view_from_str(s: &str) -> StringView<'_>;
        fn string_view_as_bytes(s: StringView<'_>) -> &[c_char];
    }
}

impl<'a> StringView<'a> {
    pub fn new(s: &'a str) -> Self {
        ffi::string_view_from_str(s)
    }

    pub fn as_bytes(self) -> &'a [c_char] {
        ffi::string_view_as_bytes(self)
    }
}

impl<'a> Deref for StringView<'a> {
    type Target = [c_char];
    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}
