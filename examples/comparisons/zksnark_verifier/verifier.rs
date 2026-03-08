
pub const PRIME_Q: u256 = 21888242871839275222246405745257275088696311157297823662689037894645226208583;
pub const SNARK_SCALAR_FIELD: u256 = 21888242871839275222246405745257275088548364400416034343698204186575808495617;

struct G1Point {
    x: u256;
    y: u256;
}

impl G1Point {
    pub fn new(x: u256, y: u256) -> Self {
        Self { x, y }
    }
    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }

    pub fn is_zero(self) -> bool {
        self.x == 0 && self.y == 0
    }

    pub fn negate(self) -> Self {
        if self.is_zero() {
            Self::zero()
        } else {
            Self::new(self.x, PRIME_Q - (self.y % PRIME_Q))
        }
    }

    pub fn plus(
        self,
        other: G1Point,
    ) -> Result<G1Point, EVMError> {
        let input: u256[4] = [self.x, self.y, other.x, other.y];
        // Call precompiled contract 0x06 (ecAdd)
        let call_res: Result<Bytes, EVMError> = EVM::static_call(0x06 as Address, ABI::encode(input));
        match call_res {
            Result::Ok(res_bytes) => ABI::decode(res_bytes),
            Result::Err(err) => Result::Err(err),
        }
    }

    pub fn scalar_mul(self, s: u256) -> Result<G1Point, EVMError> {
        let input: u256[3] = [p.x, p.y, s];

        // Call precompiled contract 0x07 (ecMul)
        let call_res: Result<Bytes, EVMError> = EVM::static_call(0x07 as Address, ABI::encode(input));
        match call_res {
            Result::Ok(res_bytes) => ABI::decode(res_bytes),
            Result::Err(err) => Result::Err(err),
        }
    }
}

// Encoding of field elements is: X[0] * z + X[1]
struct G2Point {
    x: [u256; 2];
    y: [u256; 2];
}

impl G2Point {
    pub fn new(x: [u256; 2], y: [u256; 2]) -> Self {
        Self { x, y }
    }
}

/* @return The result of computing the pairing check
    *         e(p1[0], p2[0]) *  .... * e(p1[n], p2[n]) == 1
    *         For example,
    *         pairing([P1(), P1().negate()], [P2(), P2()]) should return true.
    */
pub fn check_pairing(
    a1: G1Point,
    a2: G2Point,
    b1: G1Point,
    b2: G2Point,
    c1: G1Point,
    c2: G2Point,
    d1: G1Point,
    d2: G2Point,
) -> Result<bool, EVMError> {
    let p1: G1Point[4] = [a1, b1, c1, d1];
    let p2: G2Point[4] = [a2, b2, c2, d2];

    let input_size: u256 = 24;
    let mut input: u256[24] = [0; 24];

    for (let i = 0; i < 4; i++) {
        let j = i * 6;
        input[j + 0] = p1[i].x;
        input[j + 1] = p1[i].y;
        input[j + 2] = p2[i].x[0];
        input[j + 3] = p2[i].x[1];
        input[j + 4] = p2[i].y[0];
        input[j + 5] = p2[i].y[1];
    }

    // Call precompiled contract 0x08 (SnarkV)
    let call_res: Result<Bytes, EVMError> = EVM::static_call(0x08 as Address, ABI::encode(input));
    match call_res {
        Result::Ok(res_bytes) => match ABI::decode::<u256[1]>(res_bytes) {
            Result::Ok(out) => Result::Ok(out[0] != 0),
            Result::Err(err) => Result::Err(err),
        }
        Result::Err(err) => Result::Err(err),
    }
}

struct VerifyingKey {
    alfa1: G1Point;
    beta2: G2Point;
    gamma2: G2Point;
    delta2: G2Point;
    ic: [G1Point; 7];
}

struct Proof {
    a: G1Point;
    b: G2Point;
    c: G1Point;
}

contract Verifier {
    fn verifying_key() -> VerifyingKey {
        VerifyingKey {
            alfa1: G1Point::new(
                20692898189092739278193869274495556617788530808486270118371701516666252877969, 11713062878292653967971378194351968039596396853904572879488166084231740557279,
            ),
            beta2: G2Point::new([
                12168528810181263706895252315640534818222943348193302139358377162645029937006, 281120578337195720357474965979947690431622127986816839208576358024608803542,
            ], [
                16129176515713072042442734839012966563817890688785805090011011570989315559913, 9011703453772030375124466642203641636825223906145908770308724549646909480510,
            ]),
            gamma2: G2Point::new([
                11559732032986387107991004021392285783925812861821192530917403151452391805634,10857046999023057135944570762232829481370756359578518086990519993285655852781,
            ], [
                4082367875863433681332203403145435568316851327593401208105741076214120093531,
                8495653923123431417604973247489272438418190587263600148770280649306958101930,
            ]),
            delta2: G2Point::new([
                21280594949518992153305586783242820682644996932183186320680800072133486887432,
                150879136433974552800030963899771162647715069685890547489132178314736470662
            ], [
                1081836006956609894549771334721413187913047383331561601606260283167615953295,
                11434086686358152335540554643130007307617078324975981257823476472104616196090
            ]),
            ic: [
                G1Point::new(16225148364316337376768119297456868908427925829817748684139175309620217098814, 5167268689450204162046084442581051565997733233062478317813755636162413164690),
                G1Point::new(12882377842072682264979317445365303375159828272423495088911985689463022094260, 19488215856665173565526758360510125932214252767275816329232454875804474844786),
                G1Point::new(13083492661683431044045992285476184182144099829507350352128615182516530014777, 602051281796153692392523702676782023472744522032670801091617246498551238913),
                G1Point::new(9732465972180335629969421513785602934706096902316483580882842789662669212890, 2776526698606888434074200384264824461688198384989521091253289776235602495678),
                G1Point::new(8586364274534577154894611080234048648883781955345622578531233113180532234842, 21276134929883121123323359450658320820075698490666870487450985603988214349407),
                G1Point::new(4910628533171597675018724709631788948355422829499855033965018665300386637884, 20532468890024084510431799098097081600480376127870299142189696620752500664302),
                G1Point::new(15335858102289947642505450692012116222827233918185150176888641903531542034017, 5311597067667671581646709998171703828965875677637292315055030353779531404812),
            ],
        }
    }

    /*
     * @-> Whether the proof is valid given the hardcoded verifying key
     *          above and the public inputs
     */
    pub fn verify_proof(
        encoded_proof: Bytes,
        input: [u256; 6],
    ) -> bool {
        let p: [u256; 8] = ABI::decode::<[u256; 8]>(encoded_proof);
        // Make sure that each element in the proof is less than the prime q
        for (let i: u8 = 0; i < p.len(); i++) {
            if (p[i] >= PRIME_Q) {
                EVM::revert("verifier-proof-element-gte-prime-q");
            }
        }

        let proof: Proof = Proof {
            a: G1Point::new(p[0], p[1]);
            b: G2Point::new([p[2], p[3]], [p[4], p[5]]);
            c: G1Point::new(p[6], p[7]);
        };

        let vk: VerifyingKey = Self::verifying_key();

        // Compute the linear combination vk_x
        let mut vk_x: G1Point = G1Point::zero();
        vk_x = vk_x.plus(vk.ic[0]).unwrap();

        // Make sure that every input is less than the snark scalar field
        for (let i: u256 = 0; i < input.len(); i++) {
            if (input[i] >= SNARK_SCALAR_FIELD) {
                EVM::revert("verifier-input-gte-snark-scalar-field")
            }
            let scaled: G1Point = vk.ic[i + 1].scalar_mul(input[i]).unwrap();
            vk_x = vk_x.plus(scaled).unwrap();
        }

        check_pairing(
            proof.a.negate(),
            proof.b,
            vk.alfa1,
            vk.beta2,
            vk_x,
            vk.gamma2,
            proof.c,
            vk.delta2,
        ).unwrap()
    }
}
