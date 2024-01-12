use ic_cdk_macros::query;

#[query]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
