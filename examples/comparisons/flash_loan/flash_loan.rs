// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

abi IFlashBorrower {
    fn onFlashLoan(amount: u256);
}

contract FlashLender {
    pub debt: Mapping(address => u256);

    // 1. The Entry Point
    pub fn flashLoan(self, amount: u256)
        modifies(self.debt)
        reenters(Self::repay)
    {
        require(EVM::self_balance() >= amount, "Insufficient liquidity");

        let caller: Address = EVM::caller();
        self.debt[caller] = amount;

        modify(self.debt) {
            self.execute_transfer();
        }

        // Step A: Send the money to the borrower
        EVM::transfer(caller, amount);

        // Step B: Hand control over to the borrower
        // The borrower MUST call 'repay()' inside this function execution
        let calldata = ABI::encodeCall(IFlashBorrower::onFlashLoan, (amount));

        reenter(Self::repay) {
            EVM::raw_call(caller, calldata);
        }

        // Step C: Verify the reentrant call happened and debt is cleared
        require(self.debt[caller] == 0, "Loan not repaid via repay()");
    }

    // 2. The Reentrant Target
    // The borrower calls this function *while* flashLoan() is still executing
    pub fn repay(self)
        modifies(self.debt)
    {
        let owed: u256 = self.debt[msg.sender];
        require(owed > 0, "No active loan found");
        require(EVM::call_value() >= owed, "Insufficient repayment");

        // Clear the debt state
        self.debt[msg.sender] = 0;
    }
}
