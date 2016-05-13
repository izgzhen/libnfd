extern crate nfd_sys;
extern crate libc;

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

use std::ptr;
use std::ffi::{CString, OsStr};
use libc::size_t;
use nfd_sys::*;
use std::path::PathBuf;

#[derive(Debug)]
pub enum NFDErrorType {
    ProgrammaticError,
    CancelledByUser,
}

pub type NFDResult<T> = Result<T, NFDErrorType>;

pub fn open_dialog<'a>(filter_list: Option<&str>, default_path: Option<&str>) -> NFDResult<PathBuf> {
    unsafe {
        let out_path: *mut (*mut nfdchar_t) = &mut ptr::null_mut();

        let filter_list_ptr = match filter_list {
            Some(fl_str) => CString::new(fl_str).unwrap().into_raw(),
            None => std::ptr::null()
        };

        let default_path_ptr = match default_path {
            Some(s) => CString::new(s).unwrap().into_raw(),
            None => std::ptr::null()
        };

        match NFD_OpenDialog(filter_list_ptr, default_path_ptr, out_path) {
            nfdresult_t::NFD_ERROR => Err(NFDErrorType::ProgrammaticError),
            nfdresult_t::NFD_OKAY => {
                let bytes : Vec<u8> = CString::from_raw(*out_path).into_bytes();
                Ok(PathBuf::from(osstr_from_bytes(&bytes)))
            }
            nfdresult_t::NFD_CANCEL => Err(NFDErrorType::CancelledByUser),
        }
    }
}

#[inline]
// The only difference is in implementation: save_dialog will have an overwrite confirmation
// But the interface is same
pub fn save_dialog(filter_list: Option<&str>, default_path: Option<&str>) -> NFDResult<PathBuf> {
    open_dialog(filter_list, default_path)
}


pub fn open_dialog_multiple(filter_list: Option<&str>, default_path: Option<&str>) -> NFDResult<Vec<PathBuf>> {
    unsafe {
        let out_pathset: *mut nfdpathset_t =
            Box::into_raw(Box::new(nfdpathset_t { buf: ptr::null_mut(),
                                                  indices: ptr::null_mut(),
                                                  count: 0 }));

        let filter_list_ptr = match filter_list {
            Some(fl_str) => CString::new(fl_str).unwrap().into_raw(),
            None => std::ptr::null()
        };

        let default_path_ptr = match default_path {
            Some(s) => CString::new(s).unwrap().into_raw(),
            None => std::ptr::null()
        };

        let ret = match NFD_OpenDialogMultiple(filter_list_ptr, default_path_ptr, out_pathset) {
            nfdresult_t::NFD_ERROR  => Err(NFDErrorType::ProgrammaticError),
            nfdresult_t::NFD_OKAY   => {
                let count = NFD_PathSet_GetCount(out_pathset);
                let indices: Vec<size_t> = Vec::from_raw_parts((*out_pathset).indices, count, count);
                let mut paths = vec![];

                for i in 0..(count - 1) {
                    let ptr = NFD_PathSet_GetPath(out_pathset, indices[i]);
                    let len = indices[i + 1] - indices[i];

                    let s = String::from_raw_parts(ptr as *mut u8, len - 1, len);
                    paths.push(PathBuf::from(&s));
                }

                let ptr = (*out_pathset).buf.offset(indices[indices.len() - 1] as isize);

                let bytes : Vec<u8> = CString::from_raw(ptr).into_bytes();
                paths.push(PathBuf::from(osstr_from_bytes(&bytes)));

                NFD_PathSet_Free(out_pathset);
                Ok(paths)
            },
            nfdresult_t::NFD_CANCEL => Err(NFDErrorType::CancelledByUser),
        };
        ret
    }
}

pub fn get_error() -> String {
    unsafe {
        match CString::from_raw(NFD_GetError() as *mut nfdchar_t).into_string() {
            Ok(s) => s,
            Err(e) => panic!("{:?}", e),
        }
    }
}


fn osstr_from_bytes(slice: &[u8]) -> &OsStr {
    OsStrExt::from_bytes(slice)
}
