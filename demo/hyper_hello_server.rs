/*[toml]
[dependencies]
bytes = "1.9.0"
http-body-util = "0.1.2"
hyper = { version = "1.5.2", features = ["full"] }
pin-project-lite = "0.2.15"
pretty_env_logger = "0.5.0"
tokio = { version = "1.42.0", features = ["full"] }
*/
#![deny(warnings)]

/// Published simple hello HTTP server example from the `hyper` crate,
/// with the referenced modules `support` and `tokiort` refactored
/// into the script, while respecting their original structure and
/// redundancies.
//# Purpose: Demo `hyper` HTTP hello server, and incorporating separate modules into the script.
//# Categories: async, crates, technique
use std::convert::Infallible;
use std::net::SocketAddr;

use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use tokio::net::TcpListener;

use support::{TokioIo, TokioTimer};

// An async function that consumes a request, does nothing with it and returns a
// response.
async fn hello(_: Request<impl hyper::body::Body>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello World!"))))
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();

    // This address is localhost
    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();

    // Bind to the port and listen for incoming TCP connections
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);
    loop {
        // When an incoming TCP connection is received grab a TCP stream for
        // client<->server communication.
        //
        // Note, this is a .await point, this loop will loop forever but is not a busy loop. The
        // .await point allows the Tokio runtime to pull the task off of the thread until the task
        // has work to do. In this case, a connection arrives on the port we are listening on and
        // the task is woken up, at which point the task is then put back on a thread, and is
        // driven forward by the runtime, eventually yielding a TCP stream.
        let (tcp, _) = listener.accept().await?;
        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(tcp);

        // Spin up a new task in Tokio so we can continue to listen for new TCP connection on the
        // current task without waiting for the processing of the HTTP1 connection we just received
        // to finish
        tokio::task::spawn(async move {
            // Handle the connection from the client using HTTP1 and pass any
            // HTTP requests received on that connection to the `hello` function
            if let Err(err) = http1::Builder::new()
                .timer(TokioTimer::new())
                .serve_connection(io, service_fn(hello))
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
