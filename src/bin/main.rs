use tokio::runtime;

fn main() {
    simple_logger::init_with_level(log::Level::Debug).expect("Failed to init logging");

    let mut runtime = runtime::Builder::new()
        .threaded_scheduler()
        .thread_name("mithril-worker-pool")
        .enable_all()
        .build()
        .expect("failed to start tokio runtime");

    let handle = runtime.handle().clone();
    runtime.block_on(async move {
        mithril::main(handle).await;
    });
}
