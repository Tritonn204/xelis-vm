mod handle;

use crate::{values::ValueError, SubValue, ValuePointer};
pub use handle::{
    ValueHandle,
    ValueHandleMut
};

use super::ValueCell;

#[derive(Debug, Clone)]
pub enum Path<'a> {
    Owned(ValueCell),
    // Used for constants
    Borrowed(&'a ValueCell),
    Wrapper(ValuePointer)
}

impl<'a> Path<'a> {
    #[inline(always)]
    pub fn as_bool<'b: 'a>(&'b self) -> Result<bool, ValueError> {
        self.as_ref().as_bool()
    }

    #[inline(always)]
    pub fn as_u32(&'a self) -> Result<u32, ValueError> {
        self.as_ref().as_u32()
    }

    #[inline(always)]
    pub fn as_u64(&'a self) -> Result<u64, ValueError> {
        self.as_ref().as_u64()
    }

    pub fn shareable(&mut self) -> Path<'a> {
        match self {
            Self::Owned(v) => {
                let dst = std::mem::take(v);
                let inner = SubValue::new(dst);
                let shared = ValuePointer::shared(inner);
                *self = Self::Wrapper(shared.clone());
                Self::Wrapper(shared)
            },
            Self::Borrowed(v) => { 
                let shared = ValuePointer::shared(SubValue::new(v.clone()));
                *self = Self::Wrapper(shared.clone());
                Self::Wrapper(shared)
            },
            Self::Wrapper(v) => Self::Wrapper(v.clone())
        }
    }

    // Get the sub value of the path
    pub fn get_sub_variable(self, index: usize) -> Result<Path<'a>, ValueError> {
        match self {
            Self::Owned(v) => {
                let mut values = v.to_sub_vec()?;
                let len = values.len();
                if index >= len {
                    return Err(ValueError::OutOfBounds(index, len))
                }

                let at_index = values.remove(index);
                Ok(Path::Wrapper(at_index))
            },
            Self::Borrowed(v) => {
                let values = v.as_sub_vec()?;
                let len = values.len();
                let at_index = values
                    .get(index)
                    .ok_or_else(|| ValueError::OutOfBounds(index, len))?;

                Ok(Path::Owned(at_index.to_value()))
            },
            Self::Wrapper(mut v) => {
                let mut values = v.handle_mut();
                let values = values.as_mut_sub_vec()?;
                let len = values.len();
                let at_index = values
                    .get_mut(index)
                    .ok_or_else(|| ValueError::OutOfBounds(index, len))?;

                Ok(Path::Wrapper(at_index.transform()))
            }
        }
    }

    #[inline(always)]
    pub fn into_owned(self) -> ValueCell {
        match self {
            Self::Owned(v) => v,
            Self::Borrowed(v) => v.clone(),
            Self::Wrapper(v) => v.to_value()
        }
    }

    #[inline(always)]
    pub fn into_pointer(self) -> ValuePointer {
        match self {
            Self::Owned(v) => ValuePointer::owned(v),
            Self::Borrowed(v) => ValuePointer::owned(v.clone()),
            Self::Wrapper(v) => v.into_owned()
        }
    }

    #[inline(always)]
    pub fn as_ref<'b>(&'b self) -> ValueHandle<'b> {
        match self {
            Self::Owned(v) => ValueHandle::Borrowed(v),
            Self::Borrowed(v) => ValueHandle::Borrowed(v),
            Self::Wrapper(v) => v.handle()
        }
    }

    #[inline(always)]
    pub fn as_mut<'b>(&'b mut self) -> ValueHandleMut<'b> {
        match self {
            Self::Owned(v) => ValueHandleMut::Borrowed(v),
            Self::Borrowed(v) => {
                let v = v.clone();
                *self = Self::Owned(v);
                match self {
                    Self::Owned(v) => ValueHandleMut::Borrowed(v),
                    _ => unreachable!()
                }
            },
            Self::Wrapper(v) => v.handle_mut()
        }
    }

    // Verify if its the same pointer
    #[inline]
    pub fn is_same_ptr<'b>(&'b self, other: &'b Path<'a>) -> bool {
        self.as_ref().as_value() as *const ValueCell == other.as_ref().as_value() as *const ValueCell
    }
}