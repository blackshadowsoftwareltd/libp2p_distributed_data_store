pub mod config;
pub mod handler;
pub mod input;
pub mod run;

#[cfg(test)]
mod tests {
    use tokio::runtime::Runtime;

    use crate::run::run;

    #[test]
    fn start() {
        Runtime::new().unwrap().block_on(async move {
            run().await;
        })
    }
}
#[tokio::main]
pub async fn main() {
    run::run().await
}
