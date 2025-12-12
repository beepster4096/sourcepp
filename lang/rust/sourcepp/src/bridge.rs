use std::pin::Pin;

use cxx::{CxxString, CxxVector, UniquePtr};

#[cxx::bridge]
mod ffi {
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

        //bool hasEntry(const std::string& path, bool includeUnbaked = true) const;
        #[cxx_name = "hasEntry"]
        fn has_entry2(self: &PackFile, path: &CxxString, include_unbaked: bool) -> bool;

        type Entry;
    }

    #[cfg(feature = "vpkpp")]
    #[namespace = "vpkpp::rust_shims"]
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

    #[cfg(feature = "vpkpp")]
    #[namespace = "vpkpp::rust_shims"]
    #[repr(i32)]
    enum OpenProperty {
        DECRYPTION_KEY,
    }

    #[cfg(feature = "vpkpp")]
    #[namespace = "vpkpp::rust_shims"]
    unsafe extern "C++" {
        include!("shim/vpkpp.h");

        type c_void;

        type OpenProperty;

        fn open_packfile(path: &CxxString) -> UniquePtr<PackFile>;
        unsafe fn open_packfile_with_callbacks(
            path: &CxxString,
            callback: unsafe fn(*mut c_void, &CxxString, &Entry),
            callback_ctx: *mut c_void,
            request_property: unsafe fn(*mut c_void, *mut PackFile, OpenProperty) -> Vec<u8>,
            request_property_ctx: *mut c_void,
        ) -> UniquePtr<PackFile>;

        fn entry_data(entry: &Entry) -> EntryData<'_>;
    }
}

#[cfg(feature = "vpkpp")]
pub mod vpkpp {
    use crate::bridge::*;

    pub use ffi::{Entry, EntryData, OpenProperty, PackFile};

    impl PackFile {
        pub fn open(path: &CxxString) -> UniquePtr<PackFile> {
            ffi::open_packfile(path)
        }

        pub fn open_with_callbacks<EntryCallback, OpenPropertyRequest>(
            path: &CxxString,
            mut callback: EntryCallback,
            mut request_property: OpenPropertyRequest,
        ) -> UniquePtr<PackFile>
        where
            EntryCallback: FnMut(&CxxString, &Entry),
            OpenPropertyRequest: FnMut(Pin<&mut PackFile>, OpenProperty) -> Vec<u8>,
        {
            let callback_ctx = (&raw mut callback).cast();
            let request_property_ctx = (&raw mut request_property).cast();

            unsafe {
                ffi::open_packfile_with_callbacks(
                    path,
                    |ctx, path, entry| (*ctx.cast::<EntryCallback>())(path, entry),
                    callback_ctx,
                    |ctx, packfile, property| {
                        (*ctx.cast::<OpenPropertyRequest>())(
                            Pin::new_unchecked(&mut *packfile),
                            property,
                        )
                    },
                    request_property_ctx,
                )
            }
        }

        pub fn has_entry(&self, path: &CxxString) -> bool {
            self.has_entry2(path, true)
        }
    }

    impl Entry {
        pub fn data(&self) -> EntryData<'_> {
            ffi::entry_data(self)
        }
    }
}
