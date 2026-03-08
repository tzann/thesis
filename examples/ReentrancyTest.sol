// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

interface ERC20 {
    // We only need these functions from the standard
    function balanceOf(address account) external view returns (uint256);
    function transfer(address receiver, uint256 amount) external returns (bool);
    function transferFrom(address sender, address receiver, uint256 amount) external returns (bool);
}

contract ReentrancyTest {
    ERC20 private token;
    uint256 private ownBalance;
    mapping(address => uint256) private balances;

    // Reentrancy protection
    uint256 constant private UNLOCKED = 1;
    uint256 constant private LOCKED = 0;
    uint256 private lock = UNLOCKED;
    modifier nonReentrant() {
        require(lock == UNLOCKED);
        lock = LOCKED;
        _;
        lock = UNLOCKED;
    }

    function read() external view nonReentrant() {
        
    }

    function deposit(uint256 amount) public {
        require(token.transferFrom(msg.sender, address(this), amount));

        EVM::call(token, ABI::encode(transferFrom, msg.sender))

        uint256 newBalance = balances[msg.sender] + amount;

        balances[msg.sender] = newBalance;
    }

    function withdraw(uint256 amount) public {
        uint256 newBalance = balances[msg.sender] - amount;

        require(token.transfer(msg.sender, amount));

        balances[msg.sender] = newBalance;
    }

    function withdrawAll() public {
        uint256 amount = balances[msg.sender];

        require(
            token.transfer(msg.sender, amount)
        );

        balances[msg.sender] -= amount;
    }
}
