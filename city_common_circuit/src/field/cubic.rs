use core::fmt::Debug;

use plonky2::field::goldilocks_field::GoldilocksField;
use serde::de::DeserializeOwned;
use serde::Serialize;
use starkyx::math::extension::CubicParameters;
use starkyx::math::goldilocks::cubic::GoldilocksCubicParameters;

pub trait CubicExtendable:
    'static + Sized + Copy + Clone + Send + Sync + PartialEq + Eq + Debug + Serialize + DeserializeOwned
{
    type CubicParams: CubicParameters<Self>;
}

impl CubicExtendable for GoldilocksField {
    type CubicParams = GoldilocksCubicParameters;
}
