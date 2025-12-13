#pragma once

#include "vpkpp/vpkpp.h"
#include "rust/cxx.h"
#include <span>

namespace vpkpp {
namespace rust_shims {
    using c_void = void;

    using OpenProperty = PackFile::OpenProperty;
    using OpenPropertyRequest = rust::Fn<rust::Vec<uint8_t>(void* rustCtx, PackFile* packFile, OpenProperty property)>;

    template<typename R>
	using EntryCallbackBase = rust::Fn<R(void* rustCtx, const std::string& path, const Entry& entry)>;
	using EntryCallback     = EntryCallbackBase<void>;
	using EntryPredicate    = EntryCallbackBase<bool>;

    inline std::unique_ptr<PackFile> PackFile_open(const std::string& path) {
        return PackFile::open(path);
    }

    inline std::unique_ptr<PackFile> PackFile_open_with_callbacks(const std::string& path, EntryCallback callback, void* callbackCtx, OpenPropertyRequest requestProperty, void* requestPropertyCtx) {
        return PackFile::open(
            path,
            [=](auto path, auto entry) {
                callback(callbackCtx, path, entry);
            },
            [=](auto packFile, auto property) {
                std::vector<std::byte> cppVec;
                rust::Vec<uint8_t> rustVec = requestProperty(requestPropertyCtx, packFile, property);

                std::span<std::byte> bytes = {reinterpret_cast<std::byte*>(rustVec.data()), rustVec.size()};
                cppVec.assign(bytes.begin(), bytes.end());

                return cppVec;
            }
        );
    }

    inline std::unique_ptr<Entry> PackFile_find_entry(const PackFile& self, const std::string& path, bool includeUnbaked) {
        std::optional<Entry> entry_opt = self.findEntry(path, includeUnbaked);

        if (entry_opt) {
            return std::make_unique<Entry>(entry_opt.value());
        } else {
            return nullptr;
        }
    }

    inline rust::Vec<uint8_t> PackFile_read_entry(const PackFile& self, const std::string& path, bool& success) {
        std::optional<std::vector<std::byte>> entry_opt = self.readEntry(path);

        if (entry_opt) {
            success = true;
            auto entry_data = entry_opt.value();
            rust::Vec<uint8_t> vec;
            std::span<uint8_t> bytes = {reinterpret_cast<uint8_t*>(entry_data.data()), entry_data.size()};

            std::copy(bytes.begin(), bytes.end(), std::back_inserter(vec));

            return vec;
        } else {
            success = false;
            return {};
        }
    }

    inline void PackFile_add_entry_from_file_default(PackFile& self, const std::string& entryPath, const std::string& filepath) {
        self.addEntry(entryPath, filepath);
    }

    inline void PackFile_add_entry_from_slice(PackFile& self, const std::string& entryPath, rust::Slice<const uint8_t> slice, EntryOptions options) {
        std::span<const std::byte> bytes = {reinterpret_cast<const std::byte*>(slice.data()), slice.size()};
        self.addEntry(entryPath, bytes, options);
    }

    inline void PackFile_add_entry_from_slice_default(PackFile& self, const std::string& entryPath, rust::Slice<const uint8_t> slice) {
        PackFile_add_entry_from_slice(self, entryPath, slice, {});
    }

    // forward declared from sourcepp/src/bridge.rs.h
    struct EntryData;

    EntryData entry_data(const Entry& entry);
}
}