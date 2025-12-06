#pragma once

#include "vpkpp/vpkpp.h"
#include "rust/cxx.h"
#include <span>

namespace vpkpp {
namespace rust_shims {
    using OpenProperty = PackFile::OpenProperty;
    using OpenPropertyRequest = rust::Fn<rust::Vec<uint8_t>(void* rustCtx, PackFile* packFile, OpenProperty property)>;

    template<typename R>
	using EntryCallbackBase = rust::Fn<R(void* rustCtx, const std::string& path, const Entry& entry)>;
	using EntryCallback     = EntryCallbackBase<void>;
	using EntryPredicate    = EntryCallbackBase<bool>;

    inline std::unique_ptr<PackFile> open_packfile(const std::string& path) {
        return PackFile::open(path);
    }

    inline std::unique_ptr<PackFile> open_packfile_with_callbacks(const std::string& path, EntryCallback callback, void* callbackCtx, OpenPropertyRequest requestProperty, void* requestPropertyCtx) {
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
}
}