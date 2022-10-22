all:
	cd frontend && npm install && npm run build && cd ..
	cargo build
	sudo setcap cap_net_raw,cap_net_admin=eip target/debug/ipmap
	RUST_LOG=trace target/debug/ipmap
clean:
	rm -rf frontend/package-lock.json frontend/dist frontend/node_modules/
	cargo clean