use std::marker::PhantomData;
use std::ops::Add;

// basic size definitions

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BitUnit;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ByteUnit;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size<Unit> {
    pub value: u64,
    _marker: PhantomData<Unit>
}

impl<Unit> Size<Unit> {
    pub const fn new(value: u64) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }
    
    pub fn value(self) -> u64 {
        self.value
    }
}

impl<Unit> Add for Size<Unit> {
    type Output = Size<Unit>;
    fn add(self, rhs: Self) -> Size<Unit> {
        Size::new(self.value + rhs.value)
    }
}

pub type Bit = Size<BitUnit>;
pub type Byte = Size<ByteUnit>;

impl Byte {
    pub fn to_bits(self) -> Bit {
        Bit::new(self.value * 8)
    }
}

