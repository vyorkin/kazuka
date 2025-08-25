use alloy::primitives::{U256, address, b256, bytes};
use futures_util::StreamExt;
use kazuka_mev_share_sse::{Event, EventClient, EventTransaction};
#[cfg(test)]
use pretty_assertions::assert_eq;
use serde_json::json;
use tracing_subscriber::{
    EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

const DEFAULT_FILTER_LEVEL: &str = "trace";

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER_LEVEL));

    let _ = tracing_subscriber::registry()
        .with(fmt::layer())
        .with(env_filter)
        .try_init();
}

#[tokio::test]
async fn test_subscribe_mev_events() -> anyhow::Result<()> {
    init_tracing();

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

    let actual = events[0].as_ref().unwrap();
    let expected = Event {
        hash: b256!(
            "0xabda30c14d8a2e520028117013a68904f28eac159cdb0bca64763e80ba2edd05"
        ),
        logs: vec![],
        transactions: vec![EventTransaction {
            hash: None,
            calldata: Some(bytes!(
                "0xa9059cbb000000000000000000000000254e2535e5464e5ca932c02afc4bd76d683f500600000000000000000000000000000000000000000000079219fbb16cc8f9c000"
            )),
            function_selector: Some([0xa9, 0x05, 0x9c, 0xbb].into()),
            to: Some(address!(
                "0x57e114b691db790c35207b2e685d4a43181e6061"
            )),
            from: Some(address!(
                "0x8fef490d614fce8b93bd6f28835dd35a8b3229a9"
            )),
            value: Some(U256::from(0u64)),
            max_fee_per_gas: Some(U256::from(0x3b9aca00u64)),
            max_priority_fee_per_gas: Some(U256::from(0x3b9aca00u64)),
            nonce: Some(0x96edu64),
            chain_id: Some(1),
            access_list: None,
            gas: Some(0xd6d8u64),
            tx_type: Some(0u64),
        }],
    };
    assert_eq!(actual, &expected);

    Ok(())
}

#[tokio::test]
async fn test_subscribe_mev_events_complex() -> anyhow::Result<()> {
    init_tracing();

    let mock_server = MockServer::start().await;

    let events_data = [
        json!({
            "hash": "0xb9ac01bf34984c66c5b8891223069b5028db5089b81ddc463ce02c742d4d8ae9",
            "logs": null,
            "txs": [{
                "to": "0x1d8a691dcaf6b7d9be1942080224c18417f73ffe",
                "functionSelector": "0x00000000",
                "callData": "0x",
                "chainId": "0x1",
                "value": "0xd01868a0fe442000",
                "nonce": "0x511f",
                "maxPriorityFeePerGas": "0x12662003",
                "maxFeePerGas": "0x25f97727",
                "gas": "0x16378",
                "type": "0x2",
                "from": "0x122f90062549ff778806a15fa3b2a66973b9ea1c"
            }]
        }),
        json!({
            "hash": "0x503e8d3005f10c3a5e46861f49f1a030c6ab9fea404c60e35e1a00f5f6277c27",
            "logs": null,
            "txs": [{
                "to": "0xdac17f958d2ee523a2206206994597c13d831ec7",
                "functionSelector": "0xa9059cbb",
                "callData": "0xa9059cbb0000000000000000000000001c727a55ea3c11b0ab7d3a361fe0f3c47ce6de5d000000000000000000000000000000000000000000000000000000007d831a7a",
                "chainId": "0x1",
                "value": "0x0",
                "nonce": "0x1",
                "maxPriorityFeePerGas": "0x11170",
                "maxFeePerGas": "0x13393cec",
                "gas": "0xc6a0",
                "type": "0x2",
                "from": "0x2988e663024f2526fc340d84d3d171a9e8af98c9"
            }]
        }),
        json!({
            "hash": "0xb2d837c3fe9b2b27d9b6e99bceaf45bb3e292f42a0e9a5920de8d4da120bb737",
            "logs": null,
            "txs": [{
                "to": "0x2ddd5e07dad86af708082171ef01f5ab1fd54f3a",
                "functionSelector": "0xae4e0a18",
                "callData": "0xae4e0a1800000000000000000000000052b2f712b00f1085cd864f2cf07eb9ed0ba8078a00000000000000000000000000000000000000000000000000000000000601e40000000000000000000000000000000000000000000000000000000000000000",
                "chainId": "0x1",
                "value": "0x4ec9a54e11b000",
                "nonce": "0x66f47",
                "maxPriorityFeePerGas": "0x241886f3",
                "maxFeePerGas": "0x241886f3",
                "gas": "0x33462",
                "type": "0x0",
                "from": "0xc16157e00b1bff1522c6f01246b4fb621da048d0"
            }]
        }),
    ];

    // Join with double newlines and
    // prefix each with "data: " to simulate SSE format lines
    let sse_payload = events_data
        .iter()
        .map(|event| format!("data: {event}\n\n"))
        .collect::<Vec<_>>()
        .join("");

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

    assert_eq!(events.len(), 3);

    let event1 = events[0].as_ref().unwrap();
    let event2 = events[1].as_ref().unwrap();
    let event3 = events[2].as_ref().unwrap();

    assert_eq!(
        vec![event1, event2, event3],
        vec![
            &Event {
                hash: b256!(
                    "0xb9ac01bf34984c66c5b8891223069b5028db5089b81ddc463ce02c742d4d8ae9"
                ),
                logs: vec![],
                transactions: vec![EventTransaction {
                    hash: None,
                    calldata: Some(bytes!("0x")),
                    function_selector: Some([0x00, 0x00, 0x00, 0x00].into()),
                    to: Some(address!(
                        "0x1d8a691dcaf6b7d9be1942080224c18417f73ffe"
                    )),
                    from: Some(address!(
                        "0x122f90062549ff778806a15fa3b2a66973b9ea1c"
                    )),
                    value: Some(U256::from(0xd01868a0fe442000u64)),
                    max_fee_per_gas: Some(U256::from(0x25f97727u64)),
                    max_priority_fee_per_gas: Some(U256::from(0x12662003u64)),
                    nonce: Some(0x511fu64),
                    chain_id: Some(1),
                    access_list: None,
                    gas: Some(0x16378u64),
                    tx_type: Some(0x2u64),
                }],
            },
            &Event {
                hash: b256!(
                    "0x503e8d3005f10c3a5e46861f49f1a030c6ab9fea404c60e35e1a00f5f6277c27"
                ),
                logs: vec![],
                transactions: vec![EventTransaction {
                    hash: None,
                    calldata: Some(bytes!(
                        "0xa9059cbb0000000000000000000000001c727a55ea3c11b0ab7d3a361fe0f3c47ce6de5d000000000000000000000000000000000000000000000000000000007d831a7a"
                    )),
                    function_selector: Some([0xa9, 0x05, 0x9c, 0xbb].into()),
                    to: Some(address!(
                        "0xdac17f958d2ee523a2206206994597c13d831ec7"
                    )),
                    from: Some(address!(
                        "0x2988e663024f2526fc340d84d3d171a9e8af98c9"
                    )),
                    value: Some(U256::from(0u64)),
                    max_fee_per_gas: Some(U256::from(0x13393cecu64)),
                    max_priority_fee_per_gas: Some(U256::from(0x11170u64)),
                    nonce: Some(0x1u64),
                    chain_id: Some(1),
                    access_list: None,
                    gas: Some(0xc6a0u64),
                    tx_type: Some(0x2u64),
                }],
            },
            &Event {
                hash: b256!(
                    "0xb2d837c3fe9b2b27d9b6e99bceaf45bb3e292f42a0e9a5920de8d4da120bb737"
                ),
                logs: vec![],
                transactions: vec![EventTransaction {
                    hash: None,
                    calldata: Some(bytes!(
                        "0xae4e0a1800000000000000000000000052b2f712b00f1085cd864f2cf07eb9ed0ba8078a00000000000000000000000000000000000000000000000000000000000601e40000000000000000000000000000000000000000000000000000000000000000"
                    )),
                    function_selector: Some([0xae, 0x4e, 0x0a, 0x18].into()),
                    to: Some(address!(
                        "0x2ddd5e07dad86af708082171ef01f5ab1fd54f3a"
                    )),
                    from: Some(address!(
                        "0xc16157e00b1bff1522c6f01246b4fb621da048d0"
                    )),
                    value: Some(U256::from(0x4ec9a54e11b000u64)),
                    max_fee_per_gas: Some(U256::from(0x241886f3u64)),
                    max_priority_fee_per_gas: Some(U256::from(0x241886f3u64)),
                    nonce: Some(0x66f47u64),
                    chain_id: Some(1),
                    access_list: None,
                    gas: Some(0x33462u64),
                    tx_type: Some(0x0u64),
                }],
            },
        ]
    );

    Ok(())
}
