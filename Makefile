
build:
	@cargo build

release:
	@cargo build --release

run-serve:
	@cargo run -q -- serve

clean:
	@cargo clean
