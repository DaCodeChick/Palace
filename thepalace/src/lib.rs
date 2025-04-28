use cfg_if::cfg_if;

pub mod algo;
pub use algo::*;

pub mod prop;
pub use prop::*;

cfg_if! {
    if #[cfg(feature = "net")] {
        pub mod ext;
        pub use ext::*;

        pub mod net;
        pub use net::*;
    }
}

/// A point in the 2D space
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub v: i16,
    pub h: i16,
}

cfg_if! {
    if #[cfg(any(feature = "net", feature = "prop"))] {
        impl Point {
            pub fn from_bytes(input: &[u8], swap: bool) -> Self {
                let pt: Self;

                if swap {
                    pt = Self {
                        v: input.get_i16_ne().swap_bytes(),
                        h: input.get_i16_ne().swap_bytes(),
                    };
                } else {
                    pt = Self {
                        v: input.get_i16_ne(),
                        h: input.get_i16_ne(),
                    };
                }

                pt
            }

            pub fn to_bytes(&self, swap: bool) -> Vec<u8> {
                let mut buf = vec![];

                if swap {
                    buf.put_i16_ne(self.v.swap_bytes());
                    buf.put_i16_ne(self.h.swap_bytes());
                } else {
                    buf.put_i16_ne(self.v);
                    buf.put_i16_ne(self.h);
                }

                buf
            }
        }
    }
}
