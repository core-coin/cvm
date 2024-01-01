use crate::{B160, B176, B256, U256};
use hex_literal::hex;
use sha3::{Digest, Sha3_256};
use std::str::FromStr;

const MAINNET: &str = "cb";
const DEVIN: &str = "ab";
const PRIVATE: &str = "ce";

#[repr(u64)]
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Network {
    Mainnet = 1,
    Devin = 3,
    Private(u64),
}

impl Network {
    pub fn as_u64(&self) -> u64 {
        match self {
            Network::Mainnet => 1,
            Network::Devin => 3,
            Network::Private(n) => *n,
        }
    }

    pub fn as_u256(&self) -> U256 {
        match self {
            Network::Mainnet => U256::from(1),
            Network::Devin => U256::from(3),
            Network::Private(n) => U256::from(*n),
        }
    }

    pub fn from_prefix_numerical(prefix: u8) -> Self {
        match prefix {
            // CB
            203 => Self::Mainnet,
            // AB
            171 => Self::Devin,
            // Here we don't really care about the networkId because this function is only used in
            // ecrecover, and we only need to know what should we prefix to the address from
            // ecrecover
            _ => Self::Private(100),
        }
    }
}

impl From<u64> for Network {
    fn from(id: u64) -> Self {
        match id {
            1 => Network::Mainnet,
            3 => Network::Devin,
            n => Network::Private(n),
        }
    }
}

pub const SHA3_EMPTY: B256 = B256(hex!(
    "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a"
));

#[inline(always)]
pub fn sha3(input: &[u8]) -> B256 {
    B256::from_slice(Sha3_256::digest(input).as_slice())
}

/// Returns the address for the legacy `CREATE` scheme: [`CreateScheme::Create`]
pub fn create_address(caller: B176, nonce: u64) -> B176 {
    let mut stream = rlp::RlpStream::new_list(2);
    stream.append(&caller.0.as_ref());
    stream.append(&nonce);
    let out = sha3(&stream.out());

    // Get the last 20 bytes of the hash
    let addr = B160(out[12..].try_into().unwrap());

    // Calculate the checksum and add the network prefix
    to_ican(&addr, &Network::Mainnet)
}

/// Returns the address for the `CREATE2` scheme: [`CreateScheme::Create2`]
pub fn create2_address(caller: B176, code_hash: B256, salt: U256, network_id: u64) -> B176 {
    let network = Network::from(network_id);
    let mut hasher = Sha3_256::new();
    hasher.update([0xff]);
    hasher.update(&caller[..]);
    hasher.update(salt.to_be_bytes::<{ U256::BYTES }>());
    hasher.update(&code_hash[..]);

    // Get the last 20 bytes of the hash
    let addr = B160(hasher.finalize().as_slice()[12..].try_into().unwrap());

    // Calculate the checksum and add the network prefix
    to_ican(&addr, &network)
}

pub fn to_ican(addr: &B160, network: &Network) -> B176 {
    // Get the prefix str
    let prefix = match network {
        Network::Mainnet => MAINNET,
        Network::Devin => DEVIN,
        Network::Private(_) => PRIVATE,
    };

    // Get the number string from the hex address
    let number_str = get_number_string(addr, network);

    // Calculate the checksum
    let checksum = calculate_checksum(&number_str);

    // Format it all together
    construct_ican_address(prefix, &checksum, addr)
}

fn get_number_string(addr: &B160, network: &Network) -> String {
    let prefix = match network {
        Network::Mainnet => MAINNET,
        Network::Devin => DEVIN,
        Network::Private(_) => PRIVATE,
    };

    // We have to use the Debug trait for addr https://github.com/paritytech/parity-common/issues/656
    let mut addr_str = format!("{:?}{}{}", addr, prefix, "00");
    // Remove the 0x prefix
    addr_str = addr_str.replace("0x", "");

    // Convert every hex digit to decimal and then to String
    addr_str
        .chars()
        .map(|x| x.to_digit(16).expect("Invalid Address").to_string())
        .collect::<String>()
}

fn calculate_checksum(number_str: &str) -> u64 {
    // number_str % 97
    let result = number_str.chars().fold(0, |acc, ch| {
        let digit = ch.to_digit(10).expect("Invalid Digit") as u64;
        (acc * 10 + digit) % 97
    });

    98 - result
}

fn construct_ican_address(prefix: &str, checksum: &u64, addr: &B160) -> B176 {
    // We need to use debug for the address https://github.com/paritytech/parity-common/issues/656
    let addr = format!("{:?}", addr);
    // Remove 0x prefix
    let addr = addr.replace("0x", "");

    // If the checksum is less than 10 we need to add a zero to the address
    if *checksum < 10 {
        B176::from_str(&format!("{prefix}{zero}{checksum}{addr}", zero = "0")).unwrap()
    } else {
        B176::from_str(&format!("{prefix}{checksum}{addr}")).unwrap()
    }
}

/// Serde functions to serde as [bytes::Bytes] hex string
#[cfg(feature = "serde")]
pub mod serde_hex_bytes {
    use alloc::string::String;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S, T>(x: T, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        s.serialize_str(&alloc::format!("0x{}", hex::encode(x.as_ref())))
    }

    pub fn deserialize<'de, D>(d: D) -> Result<bytes::Bytes, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(d)?;
        if let Some(value) = value.strip_prefix("0x") {
            hex::decode(value)
        } else {
            hex::decode(&value)
        }
        .map(Into::into)
        .map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_create_one() {
        let caller = B176::from_str("cb72e8cF4629ACB360350399B6CFF367A97CF36E62B9").unwrap();
        let ican_address = create_address(caller, 1);

        assert_eq!(
            ican_address,
            B176::from_str("cb617173a16e0919885092a21559c183fd7e002bac4d").unwrap()
        );
    }
    #[test]
    fn test_create_two() {
        let caller = B176::from_str("cb72e8cF4629ACB360350399B6CFF367A97CF36E62Ba").unwrap();
        let ican_address = create_address(caller, 1);

        assert_eq!(
            ican_address,
            B176::from_str("cb138c35c28ea04cc9f9ff9a6bab2fb5e29108875ca9").unwrap()
        );
    }
    #[test]
    fn test_create_three() {
        let caller = B176::from_str("cb72e8cF4629ACB360350399B6CFF367A97CF36E62Bc").unwrap();
        let ican_address = create_address(caller, 1);

        assert_eq!(
            ican_address,
            B176::from_str("cb443ffac1268f7925441476543b00cf358c2a384768").unwrap()
        );
    }

    #[test]
    fn test_create2_one() {
        let caller = B176::from_str("cb72e8cF4629ACB360350399B6CFF367A97CF36E62B9").unwrap();
        let ican_address = create2_address(caller, B256::repeat_byte(10), U256::from(239048), 1);

        assert_eq!(
            ican_address,
            B176::from_str("cb24a85f3e83d08e1a32a6ffd94e64a55ed5fc02728a").unwrap()
        );
    }
    #[test]
    fn test_create2_two() {
        let caller = B176::from_str("cb72e8cF4629ACB360350399B6CFF367A97CF36E62Ba").unwrap();
        let ican_address = create2_address(caller, B256::repeat_byte(11), U256::from(239048), 1);

        assert_eq!(
            ican_address,
            B176::from_str("cb6525960ac8820d2ad4b789c7a0cd5ab37e9accbd6d").unwrap()
        );
    }
    #[test]
    fn test_create2_three() {
        let caller = B176::from_str("cb72e8cF4629ACB360350399B6CFF367A97CF36E62Bb").unwrap();
        let ican_address = create2_address(caller, B256::repeat_byte(12), U256::from(239048), 1);

        assert_eq!(
            ican_address,
            B176::from_str("cb987f6ed234f747b03815a9ef96a2158825bc9dffcc").unwrap()
        );
    }

    // Done
    #[test]
    fn test_get_number_string_address() {
        let address = B160::from_str("e8cF4629ACB360350399B6CFF367A97CF36E62B9").unwrap();
        let number_str = get_number_string(&address, &Network::Mainnet);
        assert_eq!(
            number_str,
            String::from("1481215462910121136035039911612151536710971215361462119121100")
        );
    }

    #[test]
    fn test_calculate_checksum_address() {
        let address = B160::from_str("e8cF4629ACB360350399B6CFF367A97CF36E62B9").unwrap();
        let number_str = get_number_string(&address, &Network::Mainnet);
        let checksum = calculate_checksum(&number_str);
        assert_eq!(checksum, 72u64);
    }
}
