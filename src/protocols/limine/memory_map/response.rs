use super::entry::LimineMemoryMapEntry;

#[repr(C)]
pub struct LimineMemoryMapResponse {
    pub revision: u64,
    pub entry_count: u64,
    pub entries: *mut *mut LimineMemoryMapEntry,
}
