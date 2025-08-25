use alloy::primitives::b256;
use futures_util::StreamExt;
use kazuka_mev_share_sse::EventClient;
use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

#[tokio::test]
async fn test_subscribe_mev_events() -> anyhow::Result<()> {
    let mock_server = MockServer::start().await;

    let event = json!({
        "hash": "0xabda30c14d8a2e520028117013a68904f28eac159cdb0bca64763e80ba2edd05",
        "logs": null,
        "txs": [{
            "to": "0x57e114b691db790c35207b2e685d4a43181e6061",
            "functionSelector": "0xa9059cbb",
            "callData": "0xa9059cbb000000000000000000000000254e2535e5464e5ca932c02afc4bd76d683f500600000000000000000000000000000000000000000000079219fbb16cc8f9c000",
            "chainId": "0x1",
            "value": "0x0",
            "nonce": "0x96ed",
            "maxPriorityFeePerGas": "0x3b9aca00",
            "maxFeePerGas": "0x3b9aca00",
            "gas": "0xd6d8",
            "type": "0x0",
            "from": "0x8fef490d614fce8b93bd6f28835dd35a8b3229a9"
        }]
    });

    let sse_payload = format!("data: {event}\n\n");

    Mock::given(method("GET"))
        .and(path("/mev-share/events"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse_payload),
        )
        .mount(&mock_server)
        .await;

    let endpoint = format!("{}/mev-share/events", mock_server.uri());
    let client = EventClient::default();
    let stream = client.events(&endpoint).await.unwrap();

    let events: Vec<_> = stream.collect().await;
    assert_eq!(events.len(), 1);

    let event = events[0].as_ref().unwrap();
    assert_eq!(
        event.hash,
        b256!(
            "0xabda30c14d8a2e520028117013a68904f28eac159cdb0bca64763e80ba2edd05"
        )
    );

    Ok(())
}
