use anchor_lang::prelude::*;

#[account]
pub struct UserRef {
    pub referrer: Pubkey, /* 32 bytes */
    pub balance: u64,     /* 8 bytes */
}

impl UserRef {
    pub const SIZE: usize = 32;
}
