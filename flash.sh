cargo build --release
arm-none-eabi-objcopy -O binary target/thumbv7m-none-eabi/release/stm32_esp stm32_esp.bin
st-flash write stm32_esp.bin 0x8000000

