enum Ownership {
    None,
    Address(Address),
    PendingTransfer(Address, Address),
}

impl Ownership {
    fn get_owner(self) -> Option<Address> {
        match self {
            Ownership::None => None,
            Ownership::Address(owner) => Some(owner),
            Ownership::PendingTransfer(owner, _) => Some(owner),
        }
    }
    fn is_owner(self, address: Address) -> bool {
        self.get_owner() == Some(address)
    }
    fn get_pending_owner(self) -> Option<Address> {
        match self {
            Ownership::None => None,
            Ownership::Address(_) => None,
            Ownership::PendingTransfer(_, pending_owner) => Some(pending_owner),
        }
    }
    fn is_pending_owner(self, address: Address) -> bool {
        self.get_pending_owner() == Some(address)
    }
}

contract Ownable {
    Ownership ownership;
}

impl Ownable {
    /**
     * @dev Initializes the contract setting the address provided by the deployer as the initial owner.
     */
    pub fn ctor(initial_owner: Address) -> Self {
        if (initial_owner == Address::ZERO) {
            EVM::revert(OwnableInvalidOwner(Address::ZERO));
        }
        Self {
            ownership: Ownership::Address(initial_owner),
        }
    }

    /**
     * @dev -> the Address of the current owner.
     */
    pub fn owner(self) -> Address {
        self.ownership.get_owner().unwrap_or(Address::ZERO)
    }

    /**
     * @dev Leaves the contract without owner. It will not be possible to call
     * `onlyOwner` fns. Can only be called by the current owner.
     *
     * NOTE: Renouncing ownership will leave the contract without an owner,
     * thereby disabling any fnality that is only available to the owner.
     */
    pub fn renounce_ownership(self) {
        if self.ownership.is_owner(EVM::caller()) {
            EVM::revert(OwnableUnauthorizedAccount(EVM::caller()));
        }
        self.execute_ownership_transfer(Address::ZERO);
    }

    /**
     * @dev Transfers ownership of the contract to a new account (`new_owner`).
     * Can only be called by the current owner.
     */
    pub fn transfer_ownership(self, new_owner: Address) {
        let owner = match self.ownership.get_owner() {
            Some(owner) => owner,
            None => EVM::revert(OwnableUnauthorizedAccount(EVM::caller())),
        };
        if owner != EVM::caller() {
            EVM::revert(OwnableUnauthorizedAccount(EVM::caller()));
        }
        if new_owner == Address::ZERO {
            EVM::revert(OwnableInvalidOwner(Address::ZERO));
        }

        self.ownership = Ownership::PendingTransfer(owner, new_owner);
        EVM::emit(OwnershipTransferStarted(owner, new_owner));
    }

    /**
     * @dev Transfers ownership of the contract to a new account (`new_owner`).
     * Internal fn without access restriction.
     */
    fn execute_ownership_transfer(self, new_owner: Address) {
        let old_owner: Address = self.ownership.get_owner().unwrap_or(Address::ZERO);
        self.ownership = Ownership::Address(new_owner);
        EVM::emit(OwnershipTransferred(old_owner, new_owner));
    }

    /**
     * @dev -> the Address of the pending owner.
     */
    pub fn pending_owner(self) -> Address {
        self.ownership.get_pending_owner().unwrap_or(Address::ZERO)
    }

    /**
     * @dev The new owner accepts the ownership transfer.
     */
    pub fn accept_ownership(self) {
        let sender: Address = EVM::caller();
        if (self.ownership.is_pending_owner(sender)) {
            EVM::revert(OwnableUnauthorizedAccount(sender));
        }
        execute_ownership_transfer(sender);
    }
}

/**
    * @dev The caller account is not authorized to perform an operation.
    */
error OwnableUnauthorizedAccount(Address account);

/**
    * @dev The owner is not a valid owner account. (eg. `Address::ZERO`)
    */
error OwnableInvalidOwner(Address owner);

event OwnershipTransferred(Address indexed previousOwner, Address indexed new_owner);
event OwnershipTransferStarted(Address indexed previousOwner, Address indexed new_owner);