use dav_server::{fakels::FakeLs, localfs::LocalFs, DavHandler};
use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    env_logger::init();
    let dir = "/tmp";
    let addr = SocketAddr::from(([127, 0, 0, 1], 4918));

    let dav_server = DavHandler::builder()
        .filesystem(LocalFs::new(dir, false, false, false))
        .locksystem(FakeLs::new())
        .build_handler();

    println!("hyper example: listening on {:?} serving {}", addr, dir);
    let listener = TcpListener::bind(addr).await.unwrap();
    loop {
        let (stream, _) = listener.accept().await.unwrap();

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);
        let dav_server = dav_server.clone();

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(|req| async {Ok::<_, Infallible>(dav_server.handle(req).await)})).await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
