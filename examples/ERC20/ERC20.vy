Transfer: event({
	_from: indexed(address),
	_to: indexed(address),
	_value: uint256
})
Approval: event({
	_owner: indexed(address),
	_spender: indexed(address),
	_value: uint256
})

name: public(bytes32)
symbol: public(bytes32)
totalSupply: public(uint256)
decimals: public(int128)
balances: public(map(address, uint256))
allowed: public(map(address, map(address, uint256)))

@public
def __init__(_name: bytes32, _symbol: bytes32, _totalSupply: uint256, _decimals: int128):
    self.name = _name
    self.symbol = _symbol
    self.totalSupply = _totalSupply
    self.decimals = _decimals

    self.balances[msg.sender] = self.totalSupply

@public
@constant
def balanceOf(_owner: address) -> uint256:
    return self.balances[_owner]

@public
def transfer(_to: address, _value: uint256) -> bool:
    assert _value <= self.balances[msg.sender], "You do not have sufficient balance to transfer these many tokens."
    assert _to != ZERO_ADDRESS, "Invalid address"

    self.balances[msg.sender] -= _value
    self.balances[_to] += _value

    log.Transfer(msg.sender, _to, _value)
    return True


@public
def transferFrom(_from: address, _to: address, _value: uint256) -> bool:
    assert _value <= self.balances[_from], "The specified account does not have sufficient balance to transfer these many tokens."
    assert _value <= self.allowed[_from][msg.sender], "You don't have approval to transfer these many tokens."
    assert _to != ZERO_ADDRESS, "Invalid address"

    self.balances[_from] -= _value
    self.allowed[_from][msg.sender] -= _value
    self.balances[_to] += _value

    log.Transfer(_from, _to, _value)
    return True

@public
def approve(_spender: address, _amount: uint256) -> bool:
    self.allowed[msg.sender][_spender] = _amount
    log.Approval(msg.sender, _spender, _amount)
    return True

@public
def increaseApproval(_spender: address, _addedValue: uint256) -> bool:
    self.allowed[msg.sender][_spender] += _addedValue
    log.Approval(msg.sender, _spender, self.allowed[msg.sender][_spender])
    return True

@public
def decreaseApproval(_spender: address, _subtractedValue: uint256) -> bool:
    if (_subtractedValue >= self.allowed[msg.sender][_spender]):
        self.allowed[msg.sender][_spender] = 0
    else:
        self.allowed[msg.sender][_spender] -= _subtractedValue

    log.Approval(msg.sender, _spender, self.allowed[msg.sender][_spender])
    return True

@public
@constant
def allowance(_owner: address, _spender: address) -> uint256:
    return self.allowed[_owner][_spender]