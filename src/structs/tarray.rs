use std::{
    mem::{self, size_of},
    slice,
};

pub struct TArray<T> {
    item_size: usize,
    raw_bytes: Vec<u8>,
    pub count: u32,
    _marker: std::marker::PhantomData<T>,
}

pub struct TArrayIter<'a, T> {
    array: &'a TArray<T>,
    index: u32,
    _marker: std::marker::PhantomData<T>,
}

impl<T> TArray<T> {
    pub fn new(raw_bytes: Vec<u8>, count: u32) -> Self {
        Self {
            item_size: size_of::<T>(),
            raw_bytes,
            count,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn iter(&self) -> TArrayIter<T> {
        TArrayIter {
            array: self,
            index: 0,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T> Iterator for TArrayIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.array.count {
            return None;
        }

        let buffer = &self.array.raw_bytes[(self.index as usize * self.array.item_size)
            ..((self.index as usize + 1) * self.array.item_size)];

        let mut item: T = unsafe { mem::zeroed() };

        unsafe {
            let item_slice =
                slice::from_raw_parts_mut(&mut item as *mut _ as *mut u8, self.array.item_size);
            item_slice.copy_from_slice(buffer);
        }

        self.index += 1;
        Some(item)
    }
}
