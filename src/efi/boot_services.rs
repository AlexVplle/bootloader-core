use super::guid::EfiGuid;
use super::table_header::EfiTableHeader;
use super::{EfiHandle, EfiStatus};

pub const EFI_LOADER_CODE: u32 = 1;
pub const EFI_LOADER_DATA: u32 = 2;

pub const EFI_ALLOCATE_ANY_PAGES: u32 = 0;
pub const EFI_ALLOCATE_MAX_ADDRESS: u32 = 1;
pub const EFI_ALLOCATE_ADDRESS: u32 = 2;

pub const EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL: u32 = 0x01;
pub const EFI_OPEN_PROTOCOL_GET_PROTOCOL: u32 = 0x02;
pub const EFI_OPEN_PROTOCOL_TEST_PROTOCOL: u32 = 0x04;
pub const EFI_OPEN_PROTOCOL_BY_CHILD_CONTROLLER: u32 = 0x08;
pub const EFI_OPEN_PROTOCOL_BY_DRIVER: u32 = 0x10;
pub const EFI_OPEN_PROTOCOL_EXCLUSIVE: u32 = 0x20;

#[repr(C)]
pub struct EfiBootServices {
    pub hdr: EfiTableHeader,
    pub raise_tpl: usize,
    pub restore_tpl: usize,
    pub allocate_pages: unsafe extern "efiapi" fn(u32, u32, usize, *mut u64) -> EfiStatus,
    pub free_pages: unsafe extern "efiapi" fn(u64, usize) -> EfiStatus,
    pub get_memory_map: unsafe extern "efiapi" fn(
        *mut usize,
        *mut u8,
        *mut usize,
        *mut usize,
        *mut u32,
    ) -> EfiStatus,
    pub allocate_pool: unsafe extern "efiapi" fn(u32, usize, *mut *mut u8) -> EfiStatus,
    pub free_pool: unsafe extern "efiapi" fn(*mut u8) -> EfiStatus,
    pub create_event: usize,
    pub set_timer: usize,
    pub wait_for_event: usize,
    pub signal_event: usize,
    pub close_event: usize,
    pub check_event: usize,
    pub install_protocol_interface: usize,
    pub reinstall_protocol_interface: usize,
    pub uninstall_protocol_interface: usize,
    pub handle_protocol:
        unsafe extern "efiapi" fn(EfiHandle, *const EfiGuid, *mut *mut u8) -> EfiStatus,
    pub _reserved: usize,
    pub register_protocol_notify: usize,
    pub locate_handle: usize,
    pub locate_device_path: usize,
    pub install_configuration_table: usize,
    pub load_image: usize,
    pub start_image: usize,
    pub exit: usize,
    pub unload_image: usize,
    pub exit_boot_services: unsafe extern "efiapi" fn(EfiHandle, usize) -> EfiStatus,
    pub get_next_monotonic_count: usize,
    pub stall: unsafe extern "efiapi" fn(usize) -> EfiStatus,
    pub set_watchdog_timer:
        unsafe extern "efiapi" fn(usize, u64, usize, *const u16) -> EfiStatus,
    pub connect_controller: usize,
    pub disconnect_controller: usize,
    pub open_protocol: unsafe extern "efiapi" fn(
        EfiHandle,
        *const EfiGuid,
        *mut *mut u8,
        EfiHandle,
        EfiHandle,
        u32,
    ) -> EfiStatus,
    pub close_protocol: usize,
    pub open_protocol_information: usize,
    pub protocols_per_handle: usize,
    pub locate_handle_buffer: usize,
    pub locate_protocol:
        unsafe extern "efiapi" fn(*const EfiGuid, *mut u8, *mut *mut u8) -> EfiStatus,
    pub install_multiple_protocol_interfaces: usize,
    pub uninstall_multiple_protocol_interfaces: usize,
    pub calculate_crc32: usize,
    pub copy_mem: usize,
    pub set_mem: usize,
    pub create_event_ex: usize,
}
