#include "shim/vpkpp.h"
#include "sourcepp/src/bridge.rs.h"

using namespace vpkpp;
using namespace rust_shims;

EntryData rust_shims::entry_data(const Entry& entry) {
    return {
        entry.flags,
        entry.archiveIndex,
        entry.length,
        entry.compressedLength,
        entry.offset,
        rust::Slice(reinterpret_cast<const uint8_t*>(entry.extraData.data()), entry.extraData.size()),
        entry.crc32,
        entry.unbaked
    };
}