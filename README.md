# WSDC Sync Job with Rust
### Requirements
- The rust compiler
- Cargo

### How to run
First, you need to set some required environment variables.
<br>
On linux or on mac os, run the following command:
```bash
export RECORD_DIRECTORY={{path_to_directory_where_to_save_records}}
export WSDC_URL=https://points.worldsdc.com/lookup2020
```
To complie the program, run the following command:
```bash
cargo build --release
```
And run it by running:
```bash
./target/release/wsdc-db-sync
```
