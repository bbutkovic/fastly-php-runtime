#[allow(unconditional_panic)]
const fn illegal_null_in_string() {
    [][0]
}

#[doc(hidden)]
pub const fn validate_cstr_contents(bytes: &[u8]) {
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\0' {
            illegal_null_in_string();
        }
        i += 1;
    }
}

macro_rules! cstr {
    ( $s:literal ) => {{
        crate::util::validate_cstr_contents($s.as_bytes());
        unsafe { std::mem::transmute::<_, &std::ffi::CStr>(concat!($s, "\0")) }
    }};
}

#[allow(unused_macros)]
macro_rules! cstring {
    ( $s:literal ) => {{
        crate::util::validate_cstr_contents($s.as_bytes());
        unsafe { std::mem::transmute::<_, &mut std::ffi::CString>(concat!($s, "\0")) }
    }};
}

pub(crate) use cstr;
#[allow(unused_imports)]
pub(crate) use cstring;
