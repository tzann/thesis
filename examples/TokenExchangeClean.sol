// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

interface ERC20 {
    // We only need these functions from the standard
    function balanceOf(address account) external view returns (uint256);
    function transfer(address receiver, uint256 amount) external returns (bool);
    function transferFrom(address sender, address receiver, uint256 amount) external returns (bool);
}

struct A {
    uint256 asdf;
}

contract Exchange {
    // Tokens and corresponding balances
    ERC20 public immutable underlyingA;
    ERC20 public immutable underlyingB;
    uint256 public balanceA;
    uint256 public balanceB;

    // Admin fees
    uint256 private constant ONE = 10**18;
    uint256 public liquidityFee;
    uint256 private accruedLiquiditySinceLastClaim;

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

    // Initialize customizable state variables
    constructor(ERC20 tokenA, ERC20 tokenB, uint256 initialLiquidityFee) {
        underlyingA = tokenA;
        underlyingB = tokenB;

        balanceA = tokenA.balanceOf(address(this));
        balanceB = tokenB.balanceOf(address(this));

        liquidityFee = initialLiquidityFee;
    }

    // +--------------------+
    // | External Interface |
    // +--------------------+

    // Side effects:
    //      May revert
    //      6 storage reads
    //      3 storage writes
    //      2 non-reentrant static calls
    //      2 non-reentrant calls
    // Technically also reads from calldata and transaction data
    function trade(uint256 amountIn, bool inverseDirection)
        external
        nonReentrant
        returns (uint256)
    {
        return _trade(amountIn, inverseDirection);
    }

    function _trade(uint256 amountIn, bool inverseDirection)
        internal
        returns (uint256)
    {
        // Side effects: 5 storage reads
        (
            ERC20 underlyingFrom,
            ERC20 underlyingTo,
            uint256 balanceFrom,
            uint256 balanceTo,
            uint256 fee
        ) = readBalances(inverseDirection);

        // Side effects: May revert, 2 non-reentrant static calls, 1 non-reentrant call
        // Technically also reads addresses from transaction data
        uint256 receivedAmount = transferFundsIn(underlyingFrom, amountIn);

        // Calculate traded amounts
        // Side effects: None (pure)
        (
            uint256 newFromBalance,
            uint256 newToBalance,
            uint256 amountOut,
            uint256 liquidityIncrease
        ) = calcTrade(receivedAmount, balanceFrom, balanceTo, fee);

        // Side effects: 1 storage read, 3 storage writes
        writeBalances(newFromBalance, newToBalance, liquidityIncrease, inverseDirection);

        // Side effects: May revert, 1 non-reentrant call
        // Technically also reads addresses from transaction data
        transferFundsOut(underlyingTo, amountOut);

        return amountOut;
    }

    // +------------------------------+
    // | Internal Effectful Functions |
    // +------------------------------+

    function readBalances(bool inverseDirection)
        internal
        view
        returns (ERC20, ERC20, uint256, uint256, uint256)
    {
        // Read from storage
        ERC20 underlyingFrom = underlyingA;
        ERC20 underlyingTo = underlyingB;
        uint256 balanceFrom = balanceA;
        uint256 balanceTo = balanceB;
        uint256 fee = liquidityFee;

        // Swap if trading in opposite direction
        if (inverseDirection) {
            (underlyingFrom, underlyingTo) = (underlyingTo, underlyingFrom);
            (balanceFrom, balanceTo) = (balanceTo, balanceFrom);
        }
        
        return (underlyingFrom, underlyingTo, balanceFrom, balanceTo, fee);
    }

    function writeBalances(uint256 newFromBalance, uint256 newToBalance, uint256 liquidityIncrease, bool inverseDirection)
        internal
    {
        (uint256 newABalance, uint256 newBBalance) = (newFromBalance, newToBalance);
        // Swap if trading in opposite direction
        if (inverseDirection) {
            (newABalance, newBBalance) = (newBBalance, newABalance);
        }

        // Write to storage
        balanceA = newABalance;
        balanceB = newBBalance;
        // Note: this incurs a read and a write to increase existing value
        accruedLiquiditySinceLastClaim += liquidityIncrease;
    }

    function transferFundsIn(ERC20 underlyingFrom, uint256 amount)
        internal
        returns (uint256)
    {
        uint256 balanceBefore = underlyingFrom.balanceOf(address(this));

        require(underlyingFrom.transferFrom(msg.sender, address(this), amount), "TransferFrom failed");

        uint256 balanceAfter = underlyingFrom.balanceOf(address(this));
        uint256 receivedAmount = balanceAfter - balanceBefore;

        return receivedAmount;
    }

    function transferFundsOut(ERC20 underlyingTo, uint256 amount)
        internal
    {
        require(underlyingTo.transfer(msg.sender, amount), "Transfer failed");
    }

    // +---------------------+
    // | Pure Math Functions |
    // +---------------------+

    function calcTrade(uint256 receivedAmount, uint256 balanceFrom, uint256 balanceTo, uint256 fee)
        internal
        pure
        returns (uint256, uint256, uint256, uint256)
    {
        // Constant product formula
        // Invariants:
        // balanceFrom * balanceTo = newToBalance * newFromBalance
        // balanceFrom + receivedAmount = newFromBalance
        uint256 product = balanceFrom * balanceTo;
        uint256 newFromBalance = balanceFrom + receivedAmount;
        uint256 newToBalance = product / newFromBalance;
        uint256 amountOut = balanceTo - newToBalance;

        // Charge fees
        uint256 feeAmount = (amountOut * fee) / ONE;
        newToBalance += feeAmount;
        amountOut -= feeAmount;

        // Remember liquidity increase for admin fees later
        uint256 liquidityBefore = sqrt(balanceFrom * balanceTo);
        uint256 liquidityAfter = sqrt(newFromBalance * newToBalance);
        uint256 liquidityIncrease = liquidityAfter - liquidityBefore;

        return (newFromBalance, newToBalance, amountOut, liquidityIncrease);
    }

    function sqrt(uint256 x) internal pure returns(uint256) {
        if (x == 0) return 0;

        // guess: 2^(log(x) / 2)
        uint256 guess = 1 << (approxLogBase2(x) >> 1);
        // 5 newton iterations because they're cheap
        // guess is provably always > 0
        guess = (guess * guess + x) / (2 * guess);
        guess = (guess * guess + x) / (2 * guess);
        guess = (guess * guess + x) / (2 * guess);
        guess = (guess * guess + x) / (2 * guess);
        guess = (guess * guess + x) / (2 * guess);
        return guess;
    }

    // sqrt(2) ≈ 886731088897/627013566048
    uint256 private constant sqrt2Numerator = 886731088897;
    uint256 private constant sqrt2Denominator = 627013566048;
    // Returns the whole number closest to the actual log base 2 of x
    function approxLogBase2(uint256 x) internal pure returns(uint256) {
        // log2(x * sqrt(2)) = log2(x) + 0.5, so truncating gives rounded value
        x = (x * sqrt2Numerator) / sqrt2Denominator;
        // highestBit is floor(log2(x))
        return highestBit(x);
    }

    // Returns the position of the highest set bit in x
    function highestBit(uint256 x) internal pure returns(uint256) {
        uint256 log = 0;
        while (0 != (x >>= 1)) {
            unchecked {
                // will never exceed 256, unchecked is fine
                log++;
            }
        }
        return log;
    }
}