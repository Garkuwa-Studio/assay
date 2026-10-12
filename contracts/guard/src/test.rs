use super::*;
use soroban_sdk::Env;

#[test]
fn version_is_zero() {
    let env = Env::default();
    let id = env.register(Guard, ());
    let client = GuardClient::new(&env, &id);
    assert_eq!(client.version(), 0);
}
