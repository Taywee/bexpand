/// Implement a Sequence IntoIterator.
/// Unfortunately, we can't just use a RangeInclusive here, because `char`
/// doesn't work, so we have to reinvent some wheels.
use std::cmp::Ordering;

pub trait CheckedAddSub: Copy + Clone {
    type Arithmetic: Copy + Clone;

    fn checked_add(self, rhs: Self::Arithmetic) -> Option<Self>;
    fn checked_sub(self, rhs: Self::Arithmetic) -> Option<Self>;
}

pub trait SequenceItem: Copy + Clone {
    /// Unsigned incrementation type.
    type Arithmetic: Copy + Clone;

    /// Arithmetic proxy type, because some types (like char) don't impl `Add`
    /// and `Sub` directly.
    type Proxy: Ord
        + Copy
        + Clone
        + From<Self>
        + TryInto<Self>
        + CheckedAddSub<Arithmetic = Self::Arithmetic>;
}

impl CheckedAddSub for isize {
    type Arithmetic = usize;
    fn checked_add(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_add_unsigned(rhs)
    }
    fn checked_sub(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_sub_unsigned(rhs)
    }
}
impl CheckedAddSub for i64 {
    type Arithmetic = u64;
    fn checked_add(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_add_unsigned(rhs)
    }
    fn checked_sub(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_sub_unsigned(rhs)
    }
}
impl CheckedAddSub for i32 {
    type Arithmetic = u32;
    fn checked_add(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_add_unsigned(rhs)
    }
    fn checked_sub(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_sub_unsigned(rhs)
    }
}
impl CheckedAddSub for i16 {
    type Arithmetic = u16;
    fn checked_add(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_add_unsigned(rhs)
    }
    fn checked_sub(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_sub_unsigned(rhs)
    }
}
impl CheckedAddSub for i8 {
    type Arithmetic = u8;
    fn checked_add(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_add_unsigned(rhs)
    }
    fn checked_sub(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_sub_unsigned(rhs)
    }
}
impl CheckedAddSub for usize {
    type Arithmetic = usize;
    fn checked_add(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_add(rhs)
    }
    fn checked_sub(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_sub(rhs)
    }
}
impl CheckedAddSub for u64 {
    type Arithmetic = u64;
    fn checked_add(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_add(rhs)
    }
    fn checked_sub(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_sub(rhs)
    }
}
impl CheckedAddSub for u32 {
    type Arithmetic = u32;
    fn checked_add(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_add(rhs)
    }
    fn checked_sub(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_sub(rhs)
    }
}
impl CheckedAddSub for u16 {
    type Arithmetic = u16;
    fn checked_add(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_add(rhs)
    }
    fn checked_sub(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_sub(rhs)
    }
}
impl CheckedAddSub for u8 {
    type Arithmetic = u8;
    fn checked_add(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_add(rhs)
    }
    fn checked_sub(self, rhs: Self::Arithmetic) -> Option<Self> {
        self.checked_sub(rhs)
    }
}

impl SequenceItem for char {
    type Arithmetic = u32;
    type Proxy = u32;
}
impl SequenceItem for isize {
    type Arithmetic = usize;
    type Proxy = isize;
}
impl SequenceItem for i64 {
    type Arithmetic = u64;
    type Proxy = i64;
}
impl SequenceItem for i32 {
    type Arithmetic = u32;
    type Proxy = i32;
}
impl SequenceItem for i16 {
    type Arithmetic = u16;
    type Proxy = i16;
}
impl SequenceItem for i8 {
    type Arithmetic = u8;
    type Proxy = i8;
}
impl SequenceItem for usize {
    type Arithmetic = usize;
    type Proxy = usize;
}
impl SequenceItem for u64 {
    type Arithmetic = u64;
    type Proxy = u64;
}
impl SequenceItem for u32 {
    type Arithmetic = u32;
    type Proxy = u32;
}
impl SequenceItem for u16 {
    type Arithmetic = u16;
    type Proxy = u16;
}
impl SequenceItem for u8 {
    type Arithmetic = u8;
    type Proxy = u8;
}

#[derive(Copy, Clone)]
pub struct Sequence<T>
where
    T: SequenceItem,
{
    pub start: T,
    pub end: T,
    pub incr: T::Arithmetic,
}

#[derive(Copy, Clone)]
pub struct SequenceIterator<T>
where
    T: SequenceItem,
{
    next: Option<T::Proxy>,
    end: T::Proxy,
    incr: T::Arithmetic,
}

impl<T> IntoIterator for Sequence<T>
where
    T: SequenceItem,
{
    type Item = Result<T, <T::Proxy as TryInto<T>>::Error>;

    type IntoIter = SequenceIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        SequenceIterator {
            next: Some(self.start.into()),
            end: self.end.into(),
            incr: self.incr,
        }
    }
}

impl<T> Iterator for SequenceIterator<T>
where
    T: SequenceItem,
{
    type Item = Result<T, <T::Proxy as TryInto<T>>::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;
        if let Some(next) = next {
            self.next = match next.cmp(&self.end) {
                Ordering::Less => {
                    // Going upwards, add incr.
                    next.checked_add(self.incr).filter(|next| next <= &self.end)
                }
                Ordering::Equal => None,
                Ordering::Greater => {
                    // Going downwards, subtract incr.
                    next.checked_sub(self.incr).filter(|next| next >= &self.end)
                }
            };
        }
        next.map(TryInto::try_into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integers() {
        let sequence = Sequence {
            start: 1i64,
            end: 10i64,
            incr: 1,
        };
        let values: Vec<_> = sequence.into_iter().collect();
        assert_eq!(
            &values,
            &[
                Ok(1i64),
                Ok(2),
                Ok(3),
                Ok(4),
                Ok(5),
                Ok(6),
                Ok(7),
                Ok(8),
                Ok(9),
                Ok(10)
            ]
        );
    }
    #[test]
    fn test_characters() {
        let sequence = Sequence {
            start: 'a',
            end: 'f',
            incr: 1,
        };
        let values: Vec<_> = sequence.into_iter().collect();
        assert_eq!(
            &values,
            &[Ok('a'), Ok('b'), Ok('c'), Ok('d'), Ok('e'), Ok('f')]
        );
    }
}
