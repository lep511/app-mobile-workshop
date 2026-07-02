use aws_sdk_dynamodb::Client as DynamoClient;
use lambda_http::{run, service_fn, tracing, Error, Request};
use users_lambda::{handle_request, AppState};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let dynamo_client = DynamoClient::new(&config);
    let state = &*Box::leak(Box::new(AppState::from_env(dynamo_client)));

    run(service_fn(|event: Request| async move {
        handle_request(event, state).await
    }))
    .await
}
