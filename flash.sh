cargo build --release
arm-none-eabi-objcopy -O binary target/thumbv7m-none-eabi/release/stm32_apa102 stm32_apa102.bin
st-flash write stm32_apa102.bin 0x8000000

