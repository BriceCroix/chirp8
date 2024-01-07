pub enum StackError {
    StackFull,  // When pushing
    StackEmpty, // When popping
}

#[cfg(feature = "alloc")]
type StackData<T, const N: usize> = alloc::vec::Vec<T>;
#[cfg(not(feature = "alloc"))]
type StackData<T, const N: usize> = [T; N];

/// Fixed-size stack storing at most `N` elements of type `T`.
pub struct Stack<T: Default + Copy, const N: usize> {
    /// Actual stack.
    data: StackData<T, N>,
    /// Where the next data is to be pushed.
    ptr: usize,
}

impl<T: Default + Copy, const N: usize> Default for Stack<T, N> {
    fn default() -> Self {

        cfg_if::cfg_if! {
            if #[cfg(feature = "alloc")]{
                let data = alloc::vec![T::default(); N];
            }else{
                let data = [T::default(); N];
            }
        }

        Self {
            data: data,
            ptr: 0,
        }
    }
}

impl<T: Default + Copy, const N: usize> Stack<T, N> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push(&mut self, value: T) -> Result<(), StackError> {
        if self.ptr < N {
            self.data[self.ptr] = value;
            self.ptr += 1;
            Ok(())
        } else {
            Err(StackError::StackFull)
        }
    }

    pub fn pop(&mut self) -> Result<T, StackError> {
        if self.ptr > 0 {
            self.ptr -= 1;
            Ok(self.data[self.ptr])
        } else {
            Err(StackError::StackEmpty)
        }
    }
}
