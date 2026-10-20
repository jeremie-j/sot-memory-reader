use std::ffi::c_void;

pub type ULONG_PTR = usize;

#[repr(C, packed)]
pub struct FName {
    pub index: u32,
    pub unknown_data_00: u32,
}

#[repr(C, packed)]
pub struct UObject {
    pub v_table_object: *mut c_void,
    pub object_flag: u32,
    pub internal_index: u32,
    pub u_class: *mut UClass,
    pub name: FName,
    pub outer: *mut Self,
}

#[repr(C, packed)]
pub struct UClass {
    pub v_table_object: u64,
    pub object_flag: u32,
    pub internal_index: u32,
    pub u_class: *const Self,
    pub name: FName,
    pub outer: *const UObject,
}

#[repr(C, packed)]
pub struct Actor {
    ptr: u64,
    id: u32,
    name: String,
}
