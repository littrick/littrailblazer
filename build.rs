include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/types/mod.rs"));
use schemars::schema_for;
use std::env;
use std::fs::File;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let schema = schema_for!(config::Config);
    serde_json::to_writer_pretty(
        File::create(out_dir.join("configlist.schema.json")).unwrap(),
        &schema,
    )
    .unwrap();
}
