use crate::{StandardPrecompileFn, Error, Precompile, PrecompileAddress, PrecompileResult};
use revm_primitives::Network;
pub const ECRECOVER: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b176(1),
    Precompile::Custom(ec_recover_run as StandardPrecompileFn),
);

#[cfg(not(feature = "secp256k1"))]
#[allow(clippy::module_inception)]
mod secp256k1 {
    use core::convert::TryFrom;
    use k256::{
        ecdsa::{recoverable, Error},
        elliptic_curve::sec1::ToEncodedPoint,
        PublicKey as K256PublicKey,
    };
    use sha3::{Digest, Keccak256};

    use crate::B256;

    pub fn ecrecover(sig: &[u8; 65], msg: &B256) -> Result<B256, Error> {
        let sig = recoverable::Signature::try_from(sig.as_ref())?;
        let verify_key = sig.recover_verifying_key_from_digest_bytes(msg.into())?;
        let public_key = K256PublicKey::from(&verify_key);
        let public_key = public_key.to_encoded_point(/* compress = */ false);
        let public_key = public_key.as_bytes();
        let hash = Keccak256::digest(&public_key[1..]);
        let mut hash: B256 = hash[..].try_into().unwrap();
        hash.iter_mut().take(12).for_each(|i| *i = 0);
        Ok(hash)
    }
}

#[cfg(feature = "secp256k1")]
#[allow(clippy::module_inception)]
mod secp256k1 {
    use crate::B256;
    use libgoldilocks::goldilocks::ed448_verify_with_error;
    use revm_primitives::{to_ican, Network, B160, B256 as rB256};
    use sha3::{Digest, Sha3_256};

    pub fn ecrecover(
        sig: &[u8; 171],
        msg: &B256,
        network: Network,
    ) -> Result<B256, libgoldilocks::errors::LibgoldilockErrors> {
        let mut sig_bytes = [0u8; 114];
        let mut pub_bytes = [0u8; 57];
        sig_bytes.copy_from_slice(&sig[0..114]);
        pub_bytes.copy_from_slice(&sig[114..171]);

        // Not sure whether this returns address(0) on invliad message
        ed448_verify_with_error(&pub_bytes, &sig_bytes, msg.as_ref())?;

        let hash = Sha3_256::digest(pub_bytes);
        let hash: B256 = hash[..].try_into().unwrap();
        let addr = B160::from_slice(&hash[12..]);
        let addr = to_ican(&addr, &network);
        let addr = rB256::from(addr);
        Ok(*addr)
    }
}

fn ec_recover_run(i: &[u8], target_energy: u64, network: Network) -> PrecompileResult {
    use core::cmp::min;

    const ECRECOVER_BASE: u64 = 3_000;

    if ECRECOVER_BASE > target_energy {
        return Err(Error::OutOfEnergy);
    }

    //                   msg+sig
    let mut input = [0u8; 32 + 171];

    // Copy the entire slice into input
    input[..min(i.len(), 171 + 32)].copy_from_slice(&i[..min(i.len(), 171 + 32)]);

    let mut msg = [0u8; 32];
    let mut sig = [0u8; 171];
    msg[0..32].copy_from_slice(&input[0..32]);
    sig[0..171].copy_from_slice(&input[32..171 + 32]);

    let out = secp256k1::ecrecover(&sig, &msg, network)
        .map(Vec::from)
        .unwrap_or_default();

    Ok((ECRECOVER_BASE, out))
}

#[cfg(test)]
mod tests {
    // use super::*;
    use crate::{
        secp256k1::{ec_recover_run, secp256k1::ecrecover},
        B256,
    };
    use hex;
    use revm_primitives::Network;

    #[test]
    fn test_recover() {
        let sig = hex::decode("611d178b128095022653965eb0ed3bc8bbea8e7891b5a121a102a5b29bb895770d204354dbbc67c5567186f92cdb58a601397dfe0022e0ce002c1333b6829c37c732fb909501f719df200ceaaa0e0a1533dc22e4c9c999406c071fee2858bc7c76c66d113ff1ac739564d465cd541b0d1e003761457fcdd53dba3dea5848c43aa54fe468284319f032945a3acb9bd4cd0fa7b7c901d978e9acd9eca43fa5b3c32b648c33dcc3f3169e8080").unwrap();
        let sig: [u8; 171] = sig.try_into().unwrap();
        let msg = hex::decode("f092a4af1f2103fe7be067df44370097c444f3bf877783ba56f21cf70ba365a3")
            .unwrap();
        let msg: [u8; 32] = msg.try_into().unwrap();
        let msg = B256::from(msg);
        let recovered = ecrecover(&sig, &msg, Network::Mainnet).unwrap();
        let expected: [u8; 32] =
            hex::decode("00000000000000000000cb58fc37a3b370a1f22e2fe2f819c210895e098845ed")
                .unwrap()
                .try_into()
                .unwrap();
        assert_eq!(recovered, expected);
    }

    #[test]
    fn test_ecrecover() {
        let sig = hex::decode("f092a4af1f2103fe7be067df44370097c444f3bf877783ba56f21cf70ba365a3611d178b128095022653965eb0ed3bc8bbea8e7891b5a121a102a5b29bb895770d204354dbbc67c5567186f92cdb58a601397dfe0022e0ce002c1333b6829c37c732fb909501f719df200ceaaa0e0a1533dc22e4c9c999406c071fee2858bc7c76c66d113ff1ac739564d465cd541b0d1e003761457fcdd53dba3dea5848c43aa54fe468284319f032945a3acb9bd4cd0fa7b7c901d978e9acd9eca43fa5b3c32b648c33dcc3f3169e8080").unwrap();
        let recovered = ec_recover_run(&sig, 5000, Network::Mainnet).unwrap().1;
        let expected: [u8; 32] =
            hex::decode("00000000000000000000cb58fc37a3b370a1f22e2fe2f819c210895e098845ed")
                .unwrap()
                .try_into()
                .unwrap();
        assert_eq!(recovered, expected);
    }
}
