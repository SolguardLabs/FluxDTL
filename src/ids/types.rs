use std::fmt;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u64);

        impl $name {
            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            pub const fn raw(self) -> u64 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "{}", self.0)
            }
        }
    };
}

id_type!(AccountId);
id_type!(AssetId);
id_type!(VaultId);
id_type!(LaneId);
id_type!(OrderId);
id_type!(EpochId);
id_type!(TxId);
