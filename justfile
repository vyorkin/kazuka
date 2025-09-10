set dotenv-load

mainnet_rpc_url := env_var("ALCHEMY_ETHEREUM_MAINNET_RPC_URL")
fork_block_number := env_var("FORK_BLOCK_NUMBER")

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
