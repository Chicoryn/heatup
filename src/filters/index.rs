use warp::Filter;

pub fn index() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::get().and(warp::fs::dir("./static"))
}
