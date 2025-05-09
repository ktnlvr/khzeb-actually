compile:
	clang -I ../../ffi/c/ --target=wasm32 -O3 -nostdlib -Wl,--no-entry -Wl,--export=get_subsystem -o main.wasm main.c
	wasm2wat main.wasm -o main.wat
