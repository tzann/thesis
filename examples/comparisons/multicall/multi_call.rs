

contract Multicall {
    // ...
}

impl Multicall {
    pub fn multicall(self, data: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        let self_addr = EVM::self_address();
        results = Vec::with_capacity(data.len());
        for (let i = 0; i < data.len(); i++) {
            reentrant(Self::*) {
                let res = EVM::delegate_call(self_addr, data[i]);
                results.push(res.unwrap());
            }
        }
        results
    }
}
