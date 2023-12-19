use libloading::{Library, Symbol};
use std::ffi::CString;
use std::os::raw::c_char;
use hex;

pub struct IceLibrary {
    ice: Library,
}

impl IceLibrary {
    pub fn new() -> Self {
        let ice = unsafe { Library::new("ice_secp256k1.dll") }.expect("Failed to load library");
        IceLibrary { ice }
    }

    pub(crate) fn init_secp256_lib(&self) {
        let init_secp256_lib: Symbol<unsafe extern "C" fn() -> ()> =
            unsafe { self.ice.get(b"init_secp256_lib") }.expect("Failed init");
        unsafe { init_secp256_lib() };
    }

    pub fn privatekey_to_address(&self, hex: &str) -> String {
        let privatekey_to_address: Symbol<unsafe extern "C" fn(i32, bool, *const c_char) -> *mut c_char> =
            unsafe { self.ice.get(b"privatekey_to_address") }.unwrap();

        let private_key = CString::new(hex).expect("Failed to create CString");
        let result = unsafe { privatekey_to_address(0, false, private_key.as_ptr()) };

        let result_str = unsafe { CString::from_raw(result) }
            .into_string()
            .expect("Failed to convert C string to String");

        result_str
    }

    pub fn privatekey_to_publickey(&self, hex: &str) -> String {
        let privatekey_to_publickey: Symbol<unsafe extern "C" fn(*const c_char, *mut u8) -> ()> =
            unsafe { self.ice.get(b"scalar_multiplication") }.unwrap();

        let private_key = CString::new(hex).expect("Failed to create CString");
        let mut res = [0u8; 65];

        unsafe { privatekey_to_publickey(private_key.as_ptr(), res.as_mut_ptr()) };

        hex::encode(res)
    }
}