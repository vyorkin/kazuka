set dotenv-load

mainnet_rpc_url := env_var("ALCHEMY_ETHEREUM_MAINNET_RPC_URL")
fork_block_number := env_var("FORK_BLOCK_NUMBER")

dry-run:
  cargo run -p kazuka-simple-arbitrage -- \
    --wss wss://eth-mainnet.g.alchemy.com/v2/of6EFa23b8xACXtwCtQ4zuYUK7YBdksL \
    --tx-signer-pk 0x81a3295e998ff7a5a0044966f094a389dc9e8600e88c7a8412f1603b6b78690d \
    --flashbots-signer-pk 0xdb01911a12111725c00a4966f222b222dc9e86da980c7a8412fa60cb6b78690d \
    --signer-pk=0x \
    --arb-contract-address 0xdAC17F958D2ee523a2206206994597C13D831ec7 \
    --dry-run

contracts:
  just forge kazuka-mev-share-arbitrage

forge-test strategy:
  forge test --etherscan-api-key $ETHERSCAN_API_KEY -vvv --watch --no-restart --root ./crates/strategies/{{strategy}}/contracts

forge-fmt strategy:
  forge fmt --root ./crates/strategies/{{strategy}}/contracts

anvil:
  anvil --fork-url {{mainnet_rpc_url}} --fork-block-number {{fork_block_number}}

chisel:
  chisel --fork-url {{mainnet_rpc_url}} --fork-block-number {{fork_block_number}}

work crate:
  cargo watch \
    -x "check -p {{crate}}" \
    -s "just test {{crate}}" \
    -s "just lint {{crate}}"

test crate:
  cargo nextest run -p {{crate}}

test-all:
  cargo nextest run --all-features --no-fail-fast --workspace

cargo-fmt:
  cargo fmt --all

lint crate:
  cargo clippy -p {{crate}}

bench-divan crate:
  cargo bench --bench {{crate}}-bench-divan >> {{crate}}.bench.divan.txt

bench-criterion crate:
  cargo bench --bench {{crate}}-bench-criterion {{crate}} >> {{crate}}.bench.criterion.txt

bench-all:
  cargo bench -q > benchmarks.txt

who-depends-on dep:
  cargo tree -e normal -i {{dep}}
