use std;
use std::borrow::{Borrow, BorrowMut};
use std::fmt;
use std::io;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr::{NonNull, null_mut};
use std::thread;
use winapi::ctypes::c_void;
use winapi::shared::minwindef::DWORD;
use winapi::shared::winerror;
use winapi::um::*;
use winapi::um::unknwnbase::IUnknown;
use winapi::um::winnt::HRESULT;

pub const E_NOTFOUND: HRESULT = hresult_from_win32_error(winerror::ERROR_NOT_FOUND);

const fn hresult_from_win32_error(x: DWORD) -> HRESULT {
    // const fn の制約を守るため、 x が 0 以下（すでに HRESULT になっている）パターンを考慮しない
    ((x & 0x0000FFFF) | ((winerror::FACILITY_WIN32 as DWORD) << 16) | 0x80000000) as HRESULT
}

pub struct SafeUnknown<T>(NonNull<T>)
    where T: Deref<Target = IUnknown>;

impl<T> Drop for SafeUnknown<T>
    where T: Deref<Target = IUnknown>
{
    fn drop(&mut self) {
        unsafe { self.Release(); }
    }
}

impl<T> Borrow<T> for SafeUnknown<T>
    where T: Deref<Target = IUnknown>
{
    fn borrow(&self) -> &T {
        unsafe { self.0.as_ref() }
    }
}

impl<T> BorrowMut<T> for SafeUnknown<T>
    where T: Deref<Target = IUnknown>
{
    fn borrow_mut(&mut self) -> &mut T {
        unsafe { self.0.as_mut() }
    }
}

impl<T> Deref for SafeUnknown<T>
    where T: Deref<Target = IUnknown>
{
    type Target = T;

    fn deref(&self) -> &T {
        self.borrow()
    }
}

impl<T> DerefMut for SafeUnknown<T>
    where T: Deref<Target = IUnknown>
{
    fn deref_mut(&mut self) -> &mut T {
        self.borrow_mut()
    }
}

impl<T> AsRef<T> for SafeUnknown<T>
    where T: Deref<Target = IUnknown>
{
    fn as_ref(&self) -> &T {
        self.borrow()
    }
}

impl<T> AsMut<T> for SafeUnknown<T>
    where T: Deref<Target = IUnknown>
{
    fn as_mut(&mut self) -> &mut T {
        self.borrow_mut()
    }
}

impl<T> Clone for SafeUnknown<T>
    where T: Deref<Target = IUnknown>
{
    fn clone(&self) -> SafeUnknown<T> {
        unsafe { self.AddRef(); }
        SafeUnknown(self.0)
    }
}

impl<T> SafeUnknown<T>
    where T: Deref<Target = IUnknown>
{
    pub unsafe fn from_non_null(ptr: NonNull<T>) -> SafeUnknown<T> {
        SafeUnknown(ptr)
    }

    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.0.as_ptr()
    }
}

impl<T> fmt::Debug for SafeUnknown<T>
    where T: Deref<Target = IUnknown>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("SafeUnknown")
            .field(&self.0)
            .finish()
    }
}

pub struct NullableSafeUnknown<T>(*mut T)
    where T: Deref<Target = IUnknown>;

impl<T> NullableSafeUnknown<T>
    where T: Deref<Target = IUnknown>
{
    pub fn null() -> NullableSafeUnknown<T> {
        NullableSafeUnknown(null_mut())
    }

    pub fn as_ptr(&self) -> *const T {
        self.0
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.0
    }

    pub fn as_void_ptr(&mut self) -> *mut c_void {
        self.0 as *mut c_void
    }

    pub fn as_mut_ref_ptr(&mut self) -> *mut *mut T {
        &mut self.0 as *mut *mut T
    }

    pub fn as_void_ref_ptr(&mut self) -> *mut *mut c_void {
        self.as_mut_ref_ptr() as *mut *mut c_void
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    pub unsafe fn not_null(mut self) -> SafeUnknown<T> {
        let result = SafeUnknown::from_non_null(
            NonNull::new(self.as_mut_ptr()).unwrap());
        mem::forget(self);
        result
    }
}

impl<T> Drop for NullableSafeUnknown<T>
    where T: Deref<Target = IUnknown>
{
    fn drop(&mut self) {
        if !self.is_null() {
            unsafe { (&mut *(self.0 as *mut IUnknown)).Release(); }
        }
    }
}

impl<T> fmt::Debug for NullableSafeUnknown<T>
    where T: Deref<Target = IUnknown>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("NullableSafeUnknown")
            .field(&self.0)
            .finish()
    }
}

#[derive(Clone, Copy)]
pub struct ComError(pub HRESULT);

impl fmt::Debug for ComError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ComError(0x{:8X})", self.0)
    }
}

impl fmt::Display for ComError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HRESULT 0x{:8X}", self.0)
    }
}

impl std::error::Error for ComError {
    fn description(&self) -> &str {
        "COM error"
    }
}

impl From<ComError> for io::Error {
    fn from(x: ComError) -> Self {
        io::Error::from_raw_os_error(x.0 as i32)
    }
}

pub type ComResult<T> = Result<T, ComError>;

pub trait HResultExt {
    fn to_result(self) -> ComResult<HRESULT>;
}

impl HResultExt for HRESULT {
    fn to_result(self) -> ComResult<HRESULT> {
        if winerror::SUCCEEDED(self) { Ok(self) }
        else { Err(ComError(self)) }
    }
}

pub fn spawn_mta_thread<F, T>(f: F) -> thread::JoinHandle<T>
    where F: FnOnce() -> T, F: Send + 'static, T: Send + 'static
{
    thread::spawn(move || {
        unsafe {
            combaseapi::CoInitializeEx(null_mut(), objbase::COINIT_MULTITHREADED)
                .to_result().unwrap();
        }

        struct ComFinalizer;
        impl Drop for ComFinalizer {
            fn drop(&mut self) {
                unsafe { combaseapi::CoUninitialize(); }
            }
        }

        let _x = ComFinalizer;

        f()
    })
}
