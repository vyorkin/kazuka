set dotenv-load

work crate:
  cargo watch -x "check -p {{crate}}" -s "just test {{crate}}" -s "just lint {{crate}}"

test crate:
  cargo nextest run -p {{crate}}

lint crate:
  cargo clippy -p {{crate}}

bench-divan crate:
  cargo bench --bench {{crate}}-bench-divan >> {{crate}}.bench.divan.txt

bench-criterion crate:
  cargo bench --bench {{crate}}-bench-criterion {{crate}} >> {{crate}}.bench.criterion.txt

bench-all:
  cargo bench -q > benchmarks.txt

