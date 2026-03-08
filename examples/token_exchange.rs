// TODO: how to integrate with solidity contracts?
//       how to handle contract interfaces in general?
interface ERC20 {
    fn balance_of(account: Address) -> u256;
    fn transfer(receiver: Address, amount: u256) -> bool;
    fn transfer_from(sender: Address, receiver: Address, amount: u256) -> bool;
}


// TODO: how to differentiate between contract definitions and structs?
struct Exchange {
    // Tokens and corresponding balances
    pub underlying_a: dyn ERC20,
    pub underlying_b: dyn ERC20,
    pub balance_a: u256,
    pub balance_b: u256,

    // Admin fees
    pub liquidity_fee: u256,
    accrued_liquidity_since_last_claim: u256,

    // Reentrancy protection not needed
}

impl Exchange {
    // TODO: how to handle contract constructors?
    pub fn constructor(underlying_a: dyn ERC20, underlying_b: dyn ERC20, liquidity_fee: u256) -> Self {
        Self {
            underlying_a,
            underlying_b,
            balance_a: underlying_a.balance_of(EVM::self_address()).unwrap(),
            balance_b: underlying_b.balance_of(EVM::self_address()).unwrap(),
            liquidity_fee,
            accrued_liquidity_since_last_claim: 0,
        }
    }

    // +--------------------+
    // | External Interface |
    // +--------------------+

    pub fn trade(&mut self, amount_in: u256, inverse_dir: bool) -> Result<u256, String>
        reads(self.underlying_a, self.underlying_b)
        reads(self.liquidity_fee)
        modifies(self.balance_a, self.balance_b)
        modifies(self.accrued_liquidity_since_last_claim)
    {
        let (
            underlying_from,
            underlying_to,
            balance_from,
            balance_to,
            fee,
        ) = self.read_balances(inverse_dir);

        let received_amount = self.transfer_funds_in(underlying_from, amount_in)?;

        // Calculate traded amounts
        let (
            new_from_balance,
            new_to_balance,
            amount_out,
            liquidity_increase,
        ) = self.calc_trade(received_amount, balance_from, balance_to, fee);
    
        self.write_balances(new_from_balance, new_to_balance, liquidity_increase, inverse_dir);

        self.transfer_funds_out(underlying_to, amount_out)?;

        amount_out
    }

    // +------------------------------+
    // | Internal Effectful Functions |
    // +------------------------------+

    fn read_balances(&self, inverse_dir: bool) -> (dyn ERC20, dyn ERC20, u256, u256, u256)
        reads(self.underlying_a, self.underlying_b)
        reads(self.balance_a, self.balance_b)
        reads(self.liquidity_fee)
    {
        let fee = self.liquidity_fee;
        if inverse_dir {
            (self.underlying_b, self.underlying_a, self.balance_b, self.balance_a, fee)
        } else {
            (self.underlying_a, self.underlying_b, self.balance_a, self.balance_b, fee)
        }
    }

    fn write_balances(&mut self, new_from_balance: u256, new_to_balance: u256, liquidity_increase: u256, inverse_dir: bool)
        modifies(self.balance_a, self.balance_b)
        modifies(self.accrued_liquidity_since_last_claim)
    {
        let new_a_balance, new_b_balance;
        if inverse_dir {
            new_a_balance = new_to_balance;
            new_b_balance = new_from_balance;
        } else {
            new_a_balance = new_from_balance;
            new_b_balance = new_to_balance;
        }

        // Write to storage
        self.balance_a = new_a_balance;
        self.balance_b = new_b_balance;
        self.accrued_liquidity_since_last_claim += liquidity_increase;
    }

    fn transfer_funds_in(&self, underlying_from: dyn ERC20, amount: u256) -> Result<u256, String> {
        let balance_before = underlying_from.balance_of(EVM::self_address()).map_err(|_| "balance_of failed".to_string())?;

        if underlying_from.transfer_from(EVM::sender(), EVM::self_address(), amount).ok().is_none_or(|succ| !succ) {
            return Err("transfer_from failed".to_string());
        }

        let balance_after = underlying_from.balance_of(EVM::self_address()).map_err(|_| "balance_of failed".to_string())?;

        let received_amount = balance_after - balance_before;
        Ok(received_amount)
    }

    fn transfer_funds_out(&self, underlying_to: dyn ERC20, amount: u256) -> Result<(), String> {
        if underlying_to.transfer(EVM::sender(), amount).ok().is_none_or(|succ| !succ) {
            Err("transfer failed".to_string())
        } else {
            Ok(())
        }
    }
}

// +---------------------+
// | Pure Math Functions |
// +---------------------+

const ONE: u256 = 10.pow(18);

// TODO: should be relatively simple, no special behavior