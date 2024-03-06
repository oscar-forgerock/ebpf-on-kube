use std::{convert::Infallible, sync::Arc};

use prometheus_client::{encoding::text::encode, registry::Registry};
use tokio::task;
use warp::{path, Filter};
mod simple;

async fn render_metrics(
    registry: Arc<tokio::sync::Mutex<Registry>>,
) -> Result<impl warp::Reply, Infallible> {
    let reg = registry.lock().await;

    let mut buffer = String::new();
    encode(&mut buffer, &reg).unwrap();
    Ok(buffer)
}

fn with_registry(
    reg: Arc<tokio::sync::Mutex<Registry>>,
) -> impl Filter<Extract = (Arc<tokio::sync::Mutex<Registry>>,), Error = std::convert::Infallible> + Clone
{
    warp::any().map(move || reg.clone())
}

#[tokio::main]
async fn main() {
    let global_registry = Arc::new(tokio::sync::Mutex::new(<Registry>::default()));
    let simple_registry = global_registry.clone();

    // run probe in the background
    task::spawn(async {
        simple::new_simple(simple_registry).await.start();
    });

    let metrics_route = path!("metrics")
        .and(with_registry(global_registry))
        .and_then(render_metrics);

    println!("Started on port 8080");

    warp::serve(metrics_route).run(([0, 0, 0, 0], 8080)).await;
    println!("END");
}