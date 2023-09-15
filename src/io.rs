pub use std::io::*;

use core::task;
use core::pin::Pin;
use hyper::rt::ReadBufCursor;

pub struct IoWrapper<T>(pub T);

impl<T: tokio::io::AsyncRead + Unpin> hyper::rt::Read for IoWrapper<T> {
    fn poll_read(self: Pin<&mut Self>, ctx: &mut task::Context<'_>, mut buf: ReadBufCursor<'_>) -> task::Poll<Result<()>> {
        let this = self.get_mut();
        let size = unsafe {
            let mut tbuf = tokio::io::ReadBuf::uninit(buf.as_mut());
            match tokio::io::AsyncRead::poll_read(Pin::new(&mut this.0), ctx, &mut tbuf) {
                task::Poll::Ready(Ok(())) => tbuf.filled().len(),
                other => return other,
            }
        };

        unsafe {
            buf.advance(size);
        }
        task::Poll::Ready(Ok(()))
    }
}

impl<T: tokio::io::AsyncWrite + Unpin> hyper::rt::Write for IoWrapper<T> {
    #[inline(always)]
    fn poll_write(self: Pin<&mut Self>, ctx: &mut task::Context<'_>, buf: &[u8]) -> task::Poll<Result<usize>> {
        let this = self.get_mut();
        tokio::io::AsyncWrite::poll_write(Pin::new(&mut this.0), ctx, buf)
    }

    #[inline(always)]
    fn poll_flush(self: Pin<&mut Self>, ctx: &mut task::Context<'_>) -> task::Poll<Result<()>> {
        let this = self.get_mut();
        tokio::io::AsyncWrite::poll_flush(Pin::new(&mut this.0), ctx)
    }

    #[inline(always)]
    fn poll_shutdown(self: Pin<&mut Self>, ctx: &mut task::Context<'_>) -> task::Poll<Result<()>> {
        let this = self.get_mut();
        tokio::io::AsyncWrite::poll_shutdown(Pin::new(&mut this.0), ctx)
    }

    #[inline(always)]
    fn is_write_vectored(&self) -> bool {
        tokio::io::AsyncWrite::is_write_vectored(&self.0)
    }

    #[inline(always)]
    fn poll_write_vectored(self: Pin<&mut Self>, ctx: &mut task::Context<'_>, bufs: &[IoSlice<'_>]) -> task::Poll<Result<usize>> {
        let this = self.get_mut();
        tokio::io::AsyncWrite::poll_write_vectored(Pin::new(&mut this.0), ctx, bufs)
    }
}
