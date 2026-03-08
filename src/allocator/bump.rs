use core::mem;
use core::ptr;

pub struct BumpAllocator {
    buffer: *mut u8,
    offset: usize,
}

impl BumpAllocator {
    pub unsafe fn new(buffer: *mut u8) -> Self {
        Self { buffer, offset: 0 }
    }

    unsafe fn alloc_raw(&mut self, size: usize, align: usize) -> *mut u8 {
        unsafe {
            self.align_to(align);
            let ptr: *mut u8 = self.ptr_at_offset(self.offset);
            self.skip(size);
            ptr
        }
    }

    pub unsafe fn alloc<T>(&mut self) -> *mut T {
        unsafe { self.alloc_raw(mem::size_of::<T>(), mem::align_of::<T>()) as *mut T }
    }

    pub unsafe fn alloc_slice<T>(&mut self, count: usize) -> *mut T {
        unsafe {
            self.alloc_raw(mem::size_of::<T>() * count, mem::align_of::<T>()) as *mut T
        }
    }

    pub unsafe fn write<T>(&mut self, value: T) {
        unsafe {
            let ptr: *mut T = self.alloc::<T>();
            ptr::write(ptr, value);
        }
    }

    pub fn skip(&mut self, count: usize) {
        self.offset += count;
    }

    pub fn align_to(&mut self, align: usize) {
        let mask: usize = align - 1;
        self.offset = (self.offset + mask) & !mask;
    }

    pub fn current_offset(&self) -> usize {
        self.offset
    }

    pub unsafe fn ptr_at_offset(&self, offset: usize) -> *mut u8 {
        unsafe { self.buffer.add(offset) }
    }
}
