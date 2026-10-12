use super::*;
use soroban_sdk::Env;

#[test]
fn version_is_zero() {
    let env = Env::default();
    let id = env.register(DemoPool, ());
    let client = DemoPoolClient::new(&env, &id);
    assert_eq!(client.version(), 0);
}
