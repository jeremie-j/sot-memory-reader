pub struct TArray<T> {
    pub ptr: *const T,
    pub count: u32,
}

pub struct TArrayIter<'a, T> {
    array: &'a TArray<T>,
    index: u32,
}

impl<T> TArray<T> {
    pub fn iter(&self) -> TArrayIter<T> {
        TArrayIter {
            array: self,
            index: 0,
        }
    }
}

impl<'a, T> Iterator for TArrayIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.array.count {
            let result = unsafe { &*self.array.ptr.offset((self.index * 8) as isize) };
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }
}
