/*[toml]
[dependencies]
hyper = { version = "1", features = ["full"] }
tokio = { version = "1", features = ["full"] }
pretty_env_logger = "0.5"
http-body-util = "0.1"
bytes = "1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
form_urlencoded = "1"
http = "1"
futures-util = { version = "0.3", default-features = false }
pin-project-lite = "0.2.14"
*/

#![deny(warnings)]

use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::Frame;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{body::Body, Method, Request, Response, StatusCode};
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::support::TokioIo;

/// Published simple echo HTTP server example from the `hyper` crate,
/// with the referenced modules `support` and `tokiort` refactored
/// into the script, while respecting their original structure and
/// redundancies.
///
/// "This is our service handler. It receives a Request, routes on its
/// path, and returns a Future of a Response."
//# Purpose: Demo `hyper` HTTP echo server, and incorporating separate modules into the script.
//# Categories: async, crates, technique
async fn echo(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") => Ok(Response::new(full(
            "Try POSTing data to /echo such as: `curl localhost:3000/echo -XPOST -d \"hello world\"`",
        ))),

        // Simply echo the body back to the client.
        (&Method::POST, "/echo") => Ok(Response::new(req.into_body().boxed())),

        // Convert to uppercase before sending back to client using a stream.
        (&Method::POST, "/echo/uppercase") => {
            let frame_stream = req.into_body().map_frame(|frame| {
                let frame = if let Ok(data) = frame.into_data() {
                    data.iter()
                        .map(|byte| byte.to_ascii_uppercase())
                        .collect::<Bytes>()
                } else {
                    Bytes::new()
                };

                Frame::data(frame)
            });

            Ok(Response::new(frame_stream.boxed()))
        }

        // Reverse the entire body before sending back to the client.
        //
        // Since we don't know the end yet, we can't simply stream
        // the chunks as they arrive as we did with the above uppercase endpoint.
        // So here we do `.await` on the future, waiting on concatenating the full body,
        // then afterwards the content can be reversed. Only then can we return a `Response`.
        (&Method::POST, "/echo/reversed") => {
            // To protect our server, reject requests with bodies larger than
            // 64kbs of data.
            let max = req.body().size_hint().upper().unwrap_or(u64::MAX);
            if max > 1024 * 64 {
                let mut resp = Response::new(full("Body too big"));
                *resp.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
                return Ok(resp);
            }

            let whole_body = req.collect().await?.to_bytes();

            let reversed_body = whole_body.iter().rev().cloned().collect::<Vec<u8>>();
            Ok(Response::new(full(reversed_body)))
        }

        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(echo))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

mod support {
    #[allow(unused)]
    pub use crate::tokiort::{TokioExecutor, TokioIo, TokioTimer};
}

mod tokiort {
    #![allow(dead_code)]
    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll},
        time::{Duration, Instant},
    };

    use hyper::rt::{Sleep, Timer};

    use pin_project_lite::pin_project;

    #[derive(Clone)]
    /// An Executor that uses the tokio runtime.
    pub struct TokioExecutor;

    impl<F> hyper::rt::Executor<F> for TokioExecutor
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        fn execute(&self, fut: F) {
            tokio::task::spawn(fut);
        }
    }

    /// A Timer that uses the tokio runtime.

    #[derive(Clone, Debug)]
    pub struct TokioTimer;

    impl Timer for TokioTimer {
        fn sleep(&self, duration: Duration) -> Pin<Box<dyn Sleep>> {
            Box::pin(TokioSleep {
                inner: tokio::time::sleep(duration),
            })
        }

        fn sleep_until(&self, deadline: Instant) -> Pin<Box<dyn Sleep>> {
            Box::pin(TokioSleep {
                inner: tokio::time::sleep_until(deadline.into()),
            })
        }

        fn reset(&self, sleep: &mut Pin<Box<dyn Sleep>>, new_deadline: Instant) {
            if let Some(sleep) = sleep.as_mut().downcast_mut_pin::<TokioSleep>() {
                sleep.reset(new_deadline.into())
            }
        }
    }

    impl TokioTimer {
        /// Create a new TokioTimer
        pub fn new() -> Self {
            Self {}
        }
    }

    // Use TokioSleep to get tokio::time::Sleep to implement Unpin.
    // see https://docs.rs/tokio/latest/tokio/time/struct.Sleep.html
    pin_project! {
        pub(crate) struct TokioSleep {
            #[pin]
            pub(crate) inner: tokio::time::Sleep,
        }
    }

    impl Future for TokioSleep {
        type Output = ();

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            self.project().inner.poll(cx)
        }
    }

    impl Sleep for TokioSleep {}

    impl TokioSleep {
        pub fn reset(self: Pin<&mut Self>, deadline: Instant) {
            self.project().inner.as_mut().reset(deadline.into());
        }
    }

    pin_project! {
        #[derive(Debug)]
        pub struct TokioIo<T> {
            #[pin]
            inner: T,
        }
    }

    impl<T> TokioIo<T> {
        pub fn new(inner: T) -> Self {
            Self { inner }
        }

        pub fn inner(self) -> T {
            self.inner
        }
    }

    impl<T> hyper::rt::Read for TokioIo<T>
    where
        T: tokio::io::AsyncRead,
    {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            mut buf: hyper::rt::ReadBufCursor<'_>,
        ) -> Poll<Result<(), std::io::Error>> {
            let n = unsafe {
                let mut tbuf = tokio::io::ReadBuf::uninit(buf.as_mut());
                match tokio::io::AsyncRead::poll_read(self.project().inner, cx, &mut tbuf) {
                    Poll::Ready(Ok(())) => tbuf.filled().len(),
                    other => return other,
                }
            };

            unsafe {
                buf.advance(n);
            }
            Poll::Ready(Ok(()))
        }
    }

    impl<T> hyper::rt::Write for TokioIo<T>
    where
        T: tokio::io::AsyncWrite,
    {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize, std::io::Error>> {
            tokio::io::AsyncWrite::poll_write(self.project().inner, cx, buf)
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), std::io::Error>> {
            tokio::io::AsyncWrite::poll_flush(self.project().inner, cx)
        }

        fn poll_shutdown(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), std::io::Error>> {
            tokio::io::AsyncWrite::poll_shutdown(self.project().inner, cx)
        }

        fn is_write_vectored(&self) -> bool {
            tokio::io::AsyncWrite::is_write_vectored(&self.inner)
        }

        fn poll_write_vectored(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            bufs: &[std::io::IoSlice<'_>],
        ) -> Poll<Result<usize, std::io::Error>> {
            tokio::io::AsyncWrite::poll_write_vectored(self.project().inner, cx, bufs)
        }
    }

    impl<T> tokio::io::AsyncRead for TokioIo<T>
    where
        T: hyper::rt::Read,
    {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            tbuf: &mut tokio::io::ReadBuf<'_>,
        ) -> Poll<Result<(), std::io::Error>> {
            //let init = tbuf.initialized().len();
            let filled = tbuf.filled().len();
            let sub_filled = unsafe {
                let mut buf = hyper::rt::ReadBuf::uninit(tbuf.unfilled_mut());

                match hyper::rt::Read::poll_read(self.project().inner, cx, buf.unfilled()) {
                    Poll::Ready(Ok(())) => buf.filled().len(),
                    other => return other,
                }
            };

            let n_filled = filled + sub_filled;
            // At least sub_filled bytes had to have been initialized.
            let n_init = sub_filled;
            unsafe {
                tbuf.assume_init(n_init);
                tbuf.set_filled(n_filled);
            }

            Poll::Ready(Ok(()))
        }
    }

    impl<T> tokio::io::AsyncWrite for TokioIo<T>
    where
        T: hyper::rt::Write,
    {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize, std::io::Error>> {
            hyper::rt::Write::poll_write(self.project().inner, cx, buf)
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), std::io::Error>> {
            hyper::rt::Write::poll_flush(self.project().inner, cx)
        }

        fn poll_shutdown(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), std::io::Error>> {
            hyper::rt::Write::poll_shutdown(self.project().inner, cx)
        }

        fn is_write_vectored(&self) -> bool {
            hyper::rt::Write::is_write_vectored(&self.inner)
        }

        fn poll_write_vectored(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            bufs: &[std::io::IoSlice<'_>],
        ) -> Poll<Result<usize, std::io::Error>> {
            hyper::rt::Write::poll_write_vectored(self.project().inner, cx, bufs)
        }
    }
}
