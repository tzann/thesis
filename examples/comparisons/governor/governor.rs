
pub enum ProposalState {
    Pending,
    Active {
        voteStart: u64,
        voteDuration: u64,
    },
    Canceled,
    Defeated,
    Succeeded,
    Queued {
        etaSeconds: u64,
    },
    Expired,
    Executed,
}
