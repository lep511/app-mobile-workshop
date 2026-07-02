use aws_sdk_dynamodb::Client as DynamoClient;
use lambda_http::{run, service_fn, Error, Request};
use users_lambda::{handle_request, AppState};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .without_time()
        .with_target(true)
        .init();

    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let dynamo_client = DynamoClient::new(&config);
    let state = &*Box::leak(Box::new(AppState::from_env(dynamo_client)));

    run(service_fn(|event: Request| async move {
        handle_request(event, state).await
    }))
    .await
}
