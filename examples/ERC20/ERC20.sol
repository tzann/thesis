// SPDX-License-Identifier: UNLICENSED

pragma solidity ^0.8.20;

contract ERC20 {
    mapping(address account => uint256) private _balances;

    mapping(address account =>
        mapping(address spender => uint256)) private _allowances;

    uint256 public totalSupply;
    uint256 public constant decimals = 18;

    string public name;

    string public symbol;
    
    constructor(string memory name_, string memory symbol_) {
        name = name_;
        symbol = symbol_;
    }

    function balanceOf(address account) external view returns (uint256) {
        return _balances[account];
    }

    function allowance(
        address owner,
        address spender
    ) external view returns (uint256) {
        return _allowances[owner][spender];
    }

    function approve(
        address spender,
        uint256 value
    ) external returns (bool) {
        require(spender != address(0), ERC20InvalidSpender(address(0)));

        address owner = msg.sender;
        _allowances[owner][spender] = value;
        emit Approval(owner, spender, value);

        return true;
    }

    function transfer(address to, uint256 value) external returns (bool) {
        require(to != address(0), ERC20InvalidReceiver(address(0)));

        address owner = msg.sender;
        _transfer(owner, to, value);

        return true;
    }

    function transferFrom(
        address from,
        address to,
        uint256 value
    ) external returns (bool) {
        require(from != address(0), ERC20InvalidSender(address(0)));
        require(to != address(0), ERC20InvalidReceiver(address(0)));
        
        address spender = msg.sender;
        uint256 currentAllowance = _allowances[from][spender];
        if (currentAllowance < type(uint256).max) {
            require(
                currentAllowance >= value,
                ERC20InsufficientAllowance(spender, currentAllowance, value)
            );
            unchecked {
                _allowances[from][spender] = currentAllowance - value;
            }
        }
        
        _transfer(from, to, value);
        return true;
    }

    function _transfer(address from, address to, uint256 value) internal {
        uint256 fromBalance = _balances[from];
        require(
            fromBalance >= value,
            ERC20InsufficientBalance(from, fromBalance, value)
        );
        
        unchecked { _balances[from] = fromBalance - value; }
        unchecked { _balances[to] += value; }
        
        emit Transfer(from, to, value);
    }

    function _mint(address account, uint256 value) internal {
        require(account != address(0), ERC20InvalidReceiver(address(0)));

        totalSupply += value; // Implicit overflow check here
        unchecked { _balances[account] += value; }

        emit Transfer(address(0), account, value);
    }

    function _burn(address account, uint256 value) internal {
        require(account != address(0), ERC20InvalidSender(address(0)));

        uint256 fromBalance = _balances[account];

        require(
            fromBalance >= value,
            ERC20InsufficientBalance(account, fromBalance, value)
        );

        unchecked { _balances[account] = fromBalance - value; }
        unchecked { totalSupply -= value; }
    }

    event Transfer(
        address indexed from,
        address indexed to,
        uint256 value
    );
    event Approval(
        address indexed owner,
        address indexed spender,
        uint256 value
    );

    error ERC20InsufficientBalance(
        address sender,
        uint256 balance,
        uint256 needed
    );
    error ERC20InvalidSender(address sender);
    error ERC20InvalidReceiver(address receiver);
    error ERC20InsufficientAllowance(
        address spender,
        uint256 allowance,
        uint256 needed
    );
    error ERC20InvalidApprover(address approver);
    error ERC20InvalidSpender(address spender);
}