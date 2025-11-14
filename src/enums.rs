#[derive(Debug, Clone, Copy)]
pub enum AuthState {
    Idle,     // 0
    Pending,  // 1
    Valid,    // 2
    Invalid,  // 3
    NetError, // 4
    NFCError, // 5
}
