mod test_contract;
mod test_env;
use crate::test_env as other_test_env;

use casper_engine_test_support::AccountHash;
pub use other_test_env::TestEnv;
pub use test_contract::TestContract;
pub struct Sender(pub AccountHash);
