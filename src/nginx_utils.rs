#![allow(dead_code)]

use nginx::*;
use std::os::raw::c_void;
use std::{mem, ptr};

use std::borrow::Cow;
use std::marker::PhantomData;
use std::slice;
use std::str::{self, Utf8Error};

pub struct NgxStr<'a>(ngx_str_t, PhantomData<&'a [u8]>);

impl<'a> NgxStr<'a> {
    pub fn new(str: &str) -> NgxStr {
        NgxStr(
            ngx_str_t {
                len: str.len() as u64,
                data: str.as_ptr() as *mut u_char,
            },
            PhantomData,
        )
    }

    pub unsafe fn from_ngx_str(str: ngx_str_t) -> NgxStr<'a> {
        NgxStr(str, PhantomData)
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.0.data, self.0.len as usize) }
    }

    pub fn to_str(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(self.as_bytes())
    }

    pub fn to_string_lossy(&self) -> Cow<str> {
        String::from_utf8_lossy(self.as_bytes())
    }

    pub fn is_empty(&self) -> bool {
        self.0.len == 0
    }
}

impl AsRef<[u8]> for NgxStr<'_> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Default for NgxStr<'_> {
    fn default() -> Self {
        NgxStr(
            ngx_str_t {
                len: 0,
                data: b"".as_ptr() as *mut u_char,
            },
            PhantomData,
        )
    }
}
#[macro_export]
macro_rules! ngx_string {
    ($x:expr) => {{
        ngx_str_t {
            len: ($x.len() - 1) as u64,
            data: $x.as_ptr() as *mut u8,
        }
    }};
}

pub trait Buffer {
    fn as_ngx_buf(&self) -> *const ngx_buf_t;

    fn as_ngx_buf_mut(&mut self) -> *mut ngx_buf_t;

    fn as_bytes(&self) -> &[u8] {
        let buf = self.as_ngx_buf();
        unsafe { slice::from_raw_parts((*buf).pos, self.len()) }
    }

    fn len(&self) -> usize {
        let buf = self.as_ngx_buf();
        unsafe {
            let pos = (*buf).pos;
            let last = (*buf).last;
            assert!(last >= pos);
            usize::wrapping_sub(last as _, pos as _)
        }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn set_last_buf(&mut self, last: bool) {
        let buf = self.as_ngx_buf_mut();
        unsafe {
            (*buf).set_last_buf(if last { 1 } else { 0 });
        }
    }

    fn set_last_in_chain(&mut self, last: bool) {
        let buf = self.as_ngx_buf_mut();
        unsafe {
            (*buf).set_last_in_chain(if last { 1 } else { 0 });
        }
    }
}

pub trait MutableBuffer: Buffer {
    fn as_bytes_mut(&mut self) -> &mut [u8] {
        let buf = self.as_ngx_buf_mut();
        unsafe { slice::from_raw_parts_mut((*buf).pos, self.len()) }
    }
}

pub struct TemporaryBuffer(*mut ngx_buf_t);

impl TemporaryBuffer {
    pub fn from_ngx_buf(buf: *mut ngx_buf_t) -> TemporaryBuffer {
        assert!(!buf.is_null());
        TemporaryBuffer(buf)
    }
}

impl Buffer for TemporaryBuffer {
    fn as_ngx_buf(&self) -> *const ngx_buf_t {
        self.0
    }

    fn as_ngx_buf_mut(&mut self) -> *mut ngx_buf_t {
        self.0
    }
}

impl MutableBuffer for TemporaryBuffer {
    fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut((*self.0).pos, self.len()) }
    }
}

pub struct MemoryBuffer(*mut ngx_buf_t);

impl MemoryBuffer {
    pub fn from_ngx_buf(buf: *mut ngx_buf_t) -> MemoryBuffer {
        assert!(!buf.is_null());
        MemoryBuffer(buf)
    }
}

impl Buffer for MemoryBuffer {
    fn as_ngx_buf(&self) -> *const ngx_buf_t {
        self.0
    }

    fn as_ngx_buf_mut(&mut self) -> *mut ngx_buf_t {
        self.0
    }
}

pub struct Pool(*mut ngx_pool_t);

impl Pool {
    pub unsafe fn from_ngx_pool(pool: *mut ngx_pool_t) -> Pool {
        assert!(!pool.is_null());
        Pool(pool)
    }

    pub fn create_buffer(&mut self, size: usize) -> Option<TemporaryBuffer> {
        let buf = unsafe { ngx_create_temp_buf(self.0, size as u64) };
        if buf.is_null() {
            return None;
        }

        Some(TemporaryBuffer::from_ngx_buf(buf))
    }

    pub fn create_buffer_from_str(&mut self, str: &str) -> Option<TemporaryBuffer> {
        let mut buffer = self.create_buffer(str.len())?;
        unsafe {
            let mut buf = buffer.as_ngx_buf_mut();
            ptr::copy_nonoverlapping(str.as_ptr(), (*buf).pos, str.len());
            (*buf).last = (*buf).pos.add(str.len());
        }
        Some(buffer)
    }

    pub fn create_buffer_from_static_str(&mut self, str: &'static str) -> Option<MemoryBuffer> {
        let buf = self.calloc_type::<ngx_buf_t>();
        if buf.is_null() {
            return None;
        }

        // We cast away const, but buffers with the memory flag are read-only
        let start = str.as_ptr() as *mut u8;
        let end = unsafe { start.add(str.len()) };

        unsafe {
            (*buf).start = start;
            (*buf).pos = start;
            (*buf).last = end;
            (*buf).end = end;
            (*buf).set_memory(1);
        }

        Some(MemoryBuffer::from_ngx_buf(buf))
    }

    unsafe fn add_cleanup_for_value<T>(&mut self, value: *mut T) -> Result<(), ()> {
        let cln = ngx_pool_cleanup_add(self.0, 0);
        if cln.is_null() {
            return Err(());
        }
        (*cln).handler = Some(cleanup_type::<T>);
        (*cln).data = value as *mut c_void;

        Ok(())
    }

    pub fn alloc(&mut self, size: usize) -> *mut c_void {
        unsafe { ngx_palloc(self.0, size as u64) }
    }

    pub fn alloc_type<T: Copy>(&mut self) -> *mut T {
        self.alloc(mem::size_of::<T>()) as *mut T
    }

    pub fn calloc(&mut self, size: usize) -> *mut c_void {
        unsafe { ngx_pcalloc(self.0, size as u64) }
    }

    pub fn calloc_type<T: Copy>(&mut self) -> *mut T {
        self.calloc(mem::size_of::<T>()) as *mut T
    }

    pub fn allocate<T>(&mut self, value: T) -> *mut T {
        unsafe {
            let p = self.alloc(mem::size_of::<T>()) as *mut T;
            ptr::write(p, value);
            if self.add_cleanup_for_value(p).is_err() {
                ptr::drop_in_place(p);
                return ptr::null_mut();
            };
            p
        }
    }
}

unsafe extern "C" fn cleanup_type<T>(data: *mut c_void) {
    ptr::drop_in_place(data as *mut T);
}
