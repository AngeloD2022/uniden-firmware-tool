# uniden-firmware-tool

As of right now, it only parses and extracts firmware binaries from https://uniden.info

## Building

Building this project requires the Rust toolchain.

```sh
git clone https://github.com/AngeloD2022/uniden-firmware-tool.git && cd uniden-firmware-tool

cargo build --release
```

## Known Issues

Many parts of the firmware BLOBs contained in most available packages from uniden.info are encrypted or encoded in some form. This has yet to be reverse engineered.

Please create an issue and let me know if you have any insights about this encryption.

## Authors

- [@AngeloD2022](https://github.com/angelod2022)
- [@jevinskie](https://github.com/jevinskie)

## License

uniden-firmware-tool is licensed under AGPLv3
