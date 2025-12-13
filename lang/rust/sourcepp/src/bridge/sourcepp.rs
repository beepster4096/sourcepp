use std::pin::Pin;

use cxx::{CxxString, CxxVector, ExternType, UniquePtr};

#[cxx::bridge]
mod ffi {
    #[namespace = "std"]
    extern "C++" {
        #[cxx_name = "string_view"]
        type StringView<'a> = crate::bridge::string_view::StringView<'a>;
    }

    #[cfg(feature = "vpkpp")]
    #[namespace = "vpkpp"]
    #[repr(i16)]
    enum EntryCompressionType {
        NO_OVERRIDE = -1,
        NO_COMPRESS = 0,
        DEFLATE = 8,
        BZIP2 = 12,
        LZMA = 14,
        ZSTD = 93,
        XZ = 95,
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

        #[cxx_name = "hasEntry"]
        fn has_entry2(self: &PackFile, path: &CxxString, include_unbaked: bool) -> bool;

        #[cxx_name = "isReadOnly"]
        fn is_read_only(self: &PackFile) -> bool;

        #[cxx_name = "addEntry"]
        fn add_entry_from_file_with_options(
            self: Pin<&mut PackFile>,
            entry_path: &CxxString,
            file_path: &CxxString,
            options: EntryOptions,
        );

        type Entry;

        type EntryCompressionType;
        type EntryOptions = super::vpkpp::EntryOptions;
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

        fn PackFile_open(path: &CxxString) -> UniquePtr<PackFile>;
        unsafe fn PackFile_open_with_callbacks(
            path: &CxxString,
            callback: unsafe fn(*mut c_void, &CxxString, &Entry),
            callback_ctx: *mut c_void,
            request_property: unsafe fn(*mut c_void, *mut PackFile, OpenProperty) -> Vec<u8>,
            request_property_ctx: *mut c_void,
        ) -> UniquePtr<PackFile>;

        fn PackFile_find_entry(
            self_: &PackFile,
            path: &CxxString,
            include_unbaked: bool,
        ) -> UniquePtr<Entry>;
        fn PackFile_read_entry(self_: &PackFile, path: &CxxString, success: &mut bool) -> Vec<u8>;
        fn PackFile_add_entry_from_file_default(
            self_: Pin<&mut PackFile>,
            entry_path: &CxxString,
            file_path: &CxxString,
        );
        fn PackFile_add_entry_from_slice(
            self_: Pin<&mut PackFile>,
            entry_path: &CxxString,
            slice: &[u8],
            options: EntryOptions,
        );
        fn PackFile_add_entry_from_slice_default(
            self_: Pin<&mut PackFile>,
            entry_path: &CxxString,
            slice: &[u8],
        );

        fn entry_data(entry: &Entry) -> EntryData<'_>;
    }
}

#[cfg(feature = "vpkpp")]
pub mod vpkpp {
    use super::*;

    pub use ffi::{Entry, EntryCompressionType, EntryData, OpenProperty, PackFile};

    impl PackFile {
        pub fn open(path: &CxxString) -> UniquePtr<PackFile> {
            ffi::PackFile_open(path)
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
                ffi::PackFile_open_with_callbacks(
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

        pub fn find_entry2(&self, path: &CxxString, include_unbaked: bool) -> UniquePtr<Entry> {
            ffi::PackFile_find_entry(self, path, include_unbaked)
        }

        pub fn find_entry(&self, path: &CxxString) -> UniquePtr<Entry> {
            self.find_entry2(path, true)
        }

        pub fn read_entry(&self, path: &CxxString) -> Option<Vec<u8>> {
            let mut success = false;
            let vec = ffi::PackFile_read_entry(self, path, &mut success);
            success.then_some(vec)
        }

        pub fn add_entry_from_file(
            self: Pin<&mut Self>,
            entry_path: &CxxString,
            file_path: &CxxString,
        ) {
            ffi::PackFile_add_entry_from_file_default(self, entry_path, file_path)
        }

        pub fn add_entry_with_options(
            self: Pin<&mut Self>,
            entry_path: &CxxString,
            slice: &[u8],
            options: EntryOptions,
        ) {
            ffi::PackFile_add_entry_from_slice(self, entry_path, slice, options)
        }

        pub fn add_entry(self: Pin<&mut Self>, entry_path: &CxxString, slice: &[u8]) {
            ffi::PackFile_add_entry_from_slice_default(self, entry_path, slice)
        }
    }

    impl Entry {
        pub fn data(&self) -> EntryData<'_> {
            ffi::entry_data(self)
        }
    }

    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct EntryOptions {
        pub zip_compressionType: EntryCompressionType,
        pub zip_compressionStrength: i16,
        pub vpk_preloadBytes: u16,
        pub vpk_saveToDirectory: bool,
    }

    unsafe impl ExternType for EntryOptions {
        type Id = cxx::type_id!("vpkpp::EntryOptions");
        type Kind = cxx::kind::Trivial;
    }
}
