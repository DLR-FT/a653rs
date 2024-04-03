use proc_macro2::TokenStream;
use quote::quote;
use syn::Path;

pub fn generate_partition_ffi(hyp: Path) -> TokenStream {
    // todo!("{hyp:?}")
    quote! {}
}

pub type APEX_INTEGER = cty::c_long;
pub type APEX_UNSIGNED = cty::c_ulong;
pub type APEX_LONG_INTEGER = cty::c_longlong;
pub type SYSTEM_TIME_TYPE = APEX_LONG_INTEGER;
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum RETURN_CODE_TYPE {
    NO_ERROR = 0,
    NO_ACTION = 1,
    NOT_AVAILABLE = 2,
    INVALID_PARAM = 3,
    INVALID_CONFIG = 4,
    INVALID_MODE = 5,
    TIMED_OUT = 6,
}
pub type LOCK_LEVEL_TYPE = APEX_INTEGER;

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum OPERATING_MODE_TYPE {
    IDLE = 0,
    COLD_START = 1,
    WARM_START = 2,
    NORMAL = 3,
}
pub type PARTITION_ID_TYPE = APEX_LONG_INTEGER;
pub type NUM_CORES_TYPE = APEX_UNSIGNED;
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum START_CONDITION_TYPE {
    NORMAL_START = 0,
    PARTITION_RESTART = 1,
    HM_MODULE_RESTART = 2,
    HM_PARTITION_RESTART = 3,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PARTITION_STATUS_TYPE {
    pub PERIOD: SYSTEM_TIME_TYPE,
    pub DURATION: SYSTEM_TIME_TYPE,
    pub IDENTIFIER: PARTITION_ID_TYPE,
    pub LOCK_LEVEL: LOCK_LEVEL_TYPE,
    pub OPERATING_MODE: OPERATING_MODE_TYPE,
    pub START_CONDITION: START_CONDITION_TYPE,
    pub NUM_ASSIGNED_CORES: NUM_CORES_TYPE,
}

extern "C" {
    pub fn GET_PARTITION_STATUS(
        PARTITION_STATUS: *mut PARTITION_STATUS_TYPE,
        RETURN_CODE: *mut RETURN_CODE_TYPE,
    );
}
extern "C" {
    pub fn SET_PARTITION_MODE(
        OPERATING_MODE: OPERATING_MODE_TYPE,
        RETURN_CODE: *mut RETURN_CODE_TYPE,
    );
}
