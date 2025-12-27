pub trait BitArray: Copy {
    fn set_bit(self, pos: u8) -> Self;

    fn clear_bit(self, pos: u8) -> Self;

    fn toggle_bit(self, pos: u8) -> Self;

    fn read_bit(self, pos: u8) -> u8;

    fn read_bitarray(self, start: u8, end: u8) -> Self;
}

macro_rules! impl_bit_array {
    ($($ty:ty),+) => {$(
        impl BitArray for $ty {
            #[inline]
            fn set_bit(self, pos: u8) -> Self {
                self | (1 << pos)
            }

            #[inline]
            fn clear_bit(self, pos: u8) -> Self {
                self & !(1 << pos)
            }

            #[inline]
            fn toggle_bit(self, pos: u8) -> Self {
                self ^ (1 << pos)
            }

            #[inline]
            fn read_bit(self, pos: u8) -> u8 {
                ((self >> pos) & 1) as u8
            }

            #[inline]
            fn read_bitarray(self, start: u8, end: u8) -> Self {
                (self >> start) & (!((!0) << (end - start)))
            }
        })+
    };
}

impl_bit_array! {u8, u16, u32, u64, u128, usize}

