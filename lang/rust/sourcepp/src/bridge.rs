use cxx::{CxxVector, CxxString, UniquePtr};

#[cxx::bridge(namespace = "vpkpp")]
mod bindings {
    #[namespace = "std"]
    extern "C++" {
        #[cxx_name = "string_view"]
        type StringView<'a> = crate::string_view::StringView<'a>;
    }

    #[cfg(feature = "vpkpp")]
    #[namespace = "vpkpp"]
    unsafe extern "C++" {
        include!("vpkpp/vpkpp.h");

        type PackFile;

        // #[Self = PackFile]
        // #[cxx_name = "getOpenableExtensions"]
        // fn get_openable_extensions() -> CxxVector<CxxString>;

        #[cxx_name = "getGUID"]
        fn get_guid(self: &PackFile) -> StringView<'_>;

        type Entry;
    }

    #[cfg(feature = "vpkpp")]
    struct EntryData<'a> {
        flags: u32,
        archive_index: u32,
        length: u64,
        compressed_length: u64,
        offset: u64,
        extra_data: &'a [u8],
        crc32: u32,
        unbaked: bool,
    }

    #[namespace = "vpkpp::rust_shims"]
    #[repr(i32)]
    enum OpenProperty {
        DECRYPTION_KEY,
    }

    #[cfg(feature = "vpkpp")]
    #[namespace = "vpkpp::rust_shims"]
    unsafe extern "C++" {
        include!("shim/vpkpp.h");

        type OpenProperty;

        fn open_packfile(path: &CxxString) -> UniquePtr<PackFile>;
        //fn open_packfile_with_callbacks(path: &CxxString, callback: unsafe fn(*mut c_void, &CxxString), request_property: &OpenPropertyRequest) -> UniquePtr<PackFile>;
    }
}

pub mod vpkpp {
    pub use super::bindings::{PackFile, OpenProperty};
}