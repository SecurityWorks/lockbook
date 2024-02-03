use tracing_subscriber::{filter, fmt, prelude::*, Layer};

pub fn init() {
    let subscriber = tracing_subscriber::Registry::default().with(
        fmt::Layer::new()
            .pretty()
            .with_target(false)
            .with_filter(filter::filter_fn(|metadata| metadata.target().starts_with("lb_fs"))),
    );

    tracing::subscriber::set_global_default(subscriber).unwrap();
}
