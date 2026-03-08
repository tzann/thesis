// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IFlashBorrower {
    function onFlashLoan(uint256 amount) external;
}

contract FlashLender {
    mapping(address => uint256) public debt;

    // Custom lock that allows specific reentrancy
    bool private _active; 

    // 1. The Entry Point
    function flashLoan(uint256 amount) external {
        require(!_active, "Loan already active");
        require(address(this).balance >= amount, "Insufficient liquidity");

        _active = true;
        debt[msg.sender] = amount;

        // Step A: Send the money to the borrower
        payable(msg.sender).transfer(amount);

        // Step B: Hand control over to the borrower
        // The borrower MUST call 'repay()' inside this function execution
        IFlashBorrower(msg.sender).onFlashLoan(amount);

        // Step C: Verify the reentrant call happened and debt is cleared
        require(debt[msg.sender] == 0, "Loan not repaid via repay()");

        _active = false;
    }

    // 2. The Reentrant Target
    // The borrower calls this function *while* flashLoan() is still executing
    function repay() external payable {
        uint256 owed = debt[msg.sender];
        require(owed > 0, "No active loan found");
        require(msg.value >= owed, "Insufficient repayment");

        // Clear the debt state
        debt[msg.sender] = 0;
    }

    // Deposit funds to the lender
    receive() external payable {}
}