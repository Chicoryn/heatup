mod filters;
mod handlers;
mod models;
mod schedule;

#[tokio::main]
async fn main() {
    warp::serve(filters::all())
        .run(([0, 0, 0, 0], 8080))
        .await;
}
