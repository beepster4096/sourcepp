#[allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use bindings::root::*;
