use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct ArcInner<T> {
  rc: AtomicUsize,
  data: T,
}

pub struct Arc<T> {
  ptr: NonNull<ArcInner<T>>,
  _marker: PhantomData<ArcInner<T>>,
}

unsafe impl<T: Send + Sync> Send for Arc<T> {}

unsafe impl<T: Send + Sync> Sync for Arc<T> {}

impl<T> Arc<T> {
  pub fn new(data: T) -> Self {
    let boxed = Box::new(
      ArcInner {
        rc: AtomicUsize::new(1),
        data,
      }
    );
    Arc {
      ptr: NonNull::new(Box::into_raw(boxed)).unwrap(),
      _marker: PhantomData,
    }
  }
}

impl<T> Deref for Arc<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    let inner = unsafe { self.ptr.as_ref() };
    &inner.data
  }
}

impl<T> Clone for Arc<T> {
  fn clone(&self) -> Self {
    let inner = unsafe { self.ptr.as_ref() };
    let old_rc = inner.rc.fetch_add(1, Ordering::Relaxed);

    if old_rc >= isize::MAX as usize {
      std::process::abort();
    }

    Self {
      ptr: self.ptr,
      _marker: PhantomData,
    }
  }
}

impl<T> Drop for Arc<T> {
  fn drop(&mut self) {
    let inner = unsafe { self.ptr.as_ref() };
    if inner.rc.fetch_add(1, Ordering::Release) != 1 {
      return;
    }
    atomic::fence(Ordering::Acquire);
    unsafe { Box::from_raw(self.ptr.as_ptr()) };
  }
}
