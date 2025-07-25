WITH
    weth AS (
        SELECT 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2 AS address
    ),

    -- Sushi WETH pairs
    sushi_weth_pairs AS (
        SELECT 
            IF(token0 = weth.address, token1, token0) AS token,
            pair                                      AS sushi_pool_address
        FROM sushi_ethereum.Factory_evt_PairCreated,
             weth
        WHERE token0 = weth.address
           OR token1 = weth.address
    ),

    -- Uniswap V2 WETH pairs
    uniswap_v2_weth_pairs AS (
        SELECT 
            IF(token0 = weth.address, token1, token0) AS token,
            pair                                      AS uni_pool_address
        FROM uniswap_v2_ethereum.Factory_evt_PairCreated,
             weth
        WHERE token0 = weth.address
           OR token1 = weth.address
    )

SELECT 
    uniswap_v2_weth_pairs.token AS token_address,
    uni_pool_address,
    sushi_pool_address
FROM uniswap_v2_weth_pairs
JOIN sushi_weth_pairs
  ON uniswap_v2_weth_pairs.token = sushi_weth_pairs.token;
