enum Allowance {
    None,
    Amount(u256),
    Unlimited,
}

impl Allowance {
    fn from_u256(amount: u256) -> Self {
        match amount {
            0 => Allowance::None,
            u256::MAX => Allowance::Unlimited,
            _ => Allowance::Amount(amount),
        }
    }
    fn to_u256(self) -> u256 {
        match self {
            Allowance::None => 0,
            Allowance::Amount(amount) => amount,
            Allowance::Unlimited => u256::MAX,
        }
    }

    fn is_unlimited(self) -> bool {
        match self {
            Allowance::None => false,
            Allowance::Amount(_) => false,
            Allowance::Unlimited => true,
        }
    }

    fn can_spend(self, amount: u256) -> bool {
        match self {
            Allowance::None => false,
            Allowance::Amount(allowance) => allowance >= amount,
            Allowance::Unlimited => true,
        }
    }

    fn spend(self, amount: u256) -> Option<Self> {
        match self {
            Allowance::None => None,
            Allowance::Amount(allowance) => {
                if allowance < amount {
                    Some(Allowance::None)
                } else {
                    Some(Allowance::Amount(allowance - amount))
                }
            }
            Allowance::Unlimited => None,
        }
    }
}

contract ERC20 {
    balances: Mapping(account: Address => u256);
    allowances: Mapping((account: Address, spender: Address) => Allowance);

    pub total_supply: u256;
    pub name: String;
    pub symbol: String;
}

impl ERC20 {
    pub const DECIMALS: u256 = 18;

    pub fn ctor(name: String, symbol: String) -> Self {
        Self {
            balances: Mapping::empty(),
            allowances: Mapping::empty(),
            total_supply: 0,
            name,
            symbol,
        }
    }

    pub fn balance_of(self, account: Address) -> u256 {
        self.balances[account]
    }

    pub fn allowance(
        self,
        owner: Address,
        spender: Address,
    ) -> u256 {
        self.allowances[owner][spender].to_u256()
    }

    pub fn approve(
        self,
        spender: Address,
        value: u256,
    ) -> bool
        modifies(self.allowances)
    {
        if spender == Address::ZERO {
            EVM::revert(ERC20InvalidSpender(spender));
        }

        let owner: Address = EVM::caller();
        self.allowances[owner][spender] = Allowance::from_u256(value);
        EVM::emit(Approval(owner, spender, value));

        true
    }

    pub fn transfer(self, to: Address, value: u256) -> bool
        modifies(self.balances)
    {
        if to == Address::ZERO {
            EVM::revert(ERC20InvalidReceiver(to));
        }

        let owner: Address = EVM::caller();
        self.execute_transfer(owner, to, value);

        true
    }

    pub fn transfer_from(
        self,
        from: Address,
        to: Address,
        value: u256,
    ) -> bool
        modifies(self.balances)
        modifies(self.allowances)
    {
        if from == Address::ZERO {
            EVM::revert(ERC20InvalidSender(from));
        }
        if to == Address::ZERO {
            EVM::revert(ERC20InvalidReceiver(to));
        }

        let spender: Address = EVM::caller();
        let allowance: Allowance = self.allowances[from][spender];

        if (!allowance.can_spend(value)) {
            EVM::revert(ERC20InsufficientAllowance(spender, currentAllowance, value));
        }
        if (!allowance.is_unlimited()) {
            self.allowances[from][spender] = allowance.spend(value);
        }

        self.execute_transfer(from, to, value);

        true
    }

    fn execute_transfer(self, from: Address, to: Address, value: u256)
        modifies(self.balances)
    {
        u256 balance = self.balances[from];
        if balance < value {
            EVM::revert(ERC20InsufficientBalance(account, balance, value));
        }
        
        self.balances[from] = balance - value;
        self.balances[to] += value;

        EVM::emit(Transfer(from, to, value));
    }

    fn _mint(self, account: Address, value: u256)
        modifies(self.balances)
    {
        if account == Address::ZERO {
            EVM::revert(ERC20InvalidReceiver(account));
        }

        self.total_supply += value;
        self.balances[account] += value;

        // Minting treated as transfer from zero address
        EVM::emit(Transfer(Address::ZERO, account, value));
    }

    fn _burn(self, account: Address, value: u256)
        modifies(self.balances)
    {
        if account == Address::ZERO {
            EVM::revert(ERC20InvalidSender(Address::ZERO));
        }

        let balance: u256 = self.balances[account];
        if balance < value {
            EVM::revert(ERC20InsufficientBalance(account, balance, value));
        }

        self.balances[account] = balance - value;
        totalSupply -= value;

        // Burning treated as transfer to zero address
        EVM::emit(Transfer(account, Address::ZERO, value));
    }
}

event Transfer(
    address indexed from,
    address indexed to,
    u256 value
);
event Approval(
    address indexed owner,
    address indexed spender,
    u256 value
);

error ERC20InsufficientBalance(
    address sender,
    u256 balance,
    u256 needed
);
error ERC20InvalidSender(address sender);
error ERC20InvalidReceiver(address receiver);
error ERC20InsufficientAllowance(
    address spender,
    u256 allowance,
    u256 needed
);
error ERC20InvalidApprover(address approver);
error ERC20InvalidSpender(address spender);
