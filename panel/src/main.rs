#[cfg(feature = "ssr")]
fn main() -> anyhow::Result<()> {
    beacon_panel::server::main()
}

#[cfg(not(feature = "ssr"))]
fn main() {
    unimplemented!()
}
