WITH
    weth AS (
        SELECT 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2 AS address
    ),

    -- Uniswap V3 WETH pools
    uniswap_v3_weth_pools AS (
        SELECT 
            IF(token0 = weth.address, token1, token0) AS token,
            IF(token0 = weth.address, TRUE, FALSE)    AS is_weth_token0,
            pool                                      AS pair,
            fee
        FROM uniswap_v3_ethereum.Factory_evt_PoolCreated,
             weth
        WHERE token0 = weth.address
           OR token1 = weth.address
    ),

    -- Uniswap V2 WETH pairs
    uniswap_v2_weth_pairs AS (
        SELECT 
            IF(token0 = weth.address, token1, token0) AS token,
            IF(token0 = weth.address, TRUE, FALSE)    AS is_weth_token0,
            pair
        FROM uniswap_v2_ethereum.Factory_evt_PairCreated,
             weth
        WHERE token0 = weth.address
           OR token1 = weth.address
    )

SELECT 
    uniswap_v3_weth_pools.token AS token_address,
    uniswap_v3_weth_pools.pair  AS v3_pool,
    uniswap_v2_weth_pairs.pair  AS v2_pool,
    uniswap_v3_weth_pools.is_weth_token0
FROM uniswap_v3_weth_pools
JOIN uniswap_v2_weth_pairs
  ON uniswap_v3_weth_pools.token = uniswap_v2_weth_pairs.token;
