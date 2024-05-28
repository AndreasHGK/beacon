use std::{
    io::{self, Cursor, Read, Write},
    task::Poll,
};

use anyhow::anyhow;
use tokio::{io::AsyncRead, sync::mpsc};

/// Creates a new byte stream with a sending and a receiving half.
///
/// A buffer size can be specified to indicate the maximum amount of bytes that
/// should be buffered before flushing automatically. This size is a lower bound
/// like in [Vec::with_capacity].
pub fn byte_stream(buffer_size: usize) -> (ByteWriter, ByteReader) {
    let (sender, receiver) = mpsc::channel(1);

    (
        ByteWriter {
            sender,
            buffer: Vec::with_capacity(buffer_size),
        },
        ByteReader {
            receiver,
            buffer: Default::default(),
        },
    )
}

/// Writing half of the byte stream.
pub struct ByteWriter {
    sender: mpsc::Sender<Vec<u8>>,
    buffer: Vec<u8>,
}

impl Write for ByteWriter {
    fn write(&mut self, original_buf: &[u8]) -> io::Result<usize> {
        let debug_capacity = self.buffer.capacity();

        let remaining_buffer_cap = self.buffer.capacity() - self.buffer.len();
        let write_amount = remaining_buffer_cap.min(original_buf.len());
        let write_amount = self.buffer.write(&original_buf[..write_amount])?;

        let buf = &original_buf[write_amount..];
        // There were more bytes in `buf` than can fit in our own buffer. First flush the now
        // full buffer and then send the remaining bytes in one go.
        if !buf.is_empty() {
            self.flush()?;
            self.sender
                .blocking_send(buf.to_owned())
                .map_err(io::Error::other)?;
        }

        // We don't want the buffer to grow.
        debug_assert_eq!(debug_capacity, self.buffer.capacity());

        Ok(original_buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        self.sender
            .blocking_send(self.buffer.as_slice().to_owned())
            .map_err(io::Error::other)?;
        self.buffer.clear();
        Ok(())
    }
}

impl Drop for ByteWriter {
    fn drop(&mut self) {
        _ = self.flush();
    }
}

/// Reading half of the byte stream.
pub struct ByteReader {
    receiver: mpsc::Receiver<Vec<u8>>,
    buffer: Cursor<Vec<u8>>,
}

impl Read for ByteReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.buffer.position() == self.buffer.get_ref().len() as u64 {
            self.buffer = Cursor::new(self.receiver.blocking_recv().ok_or_else(|| {
                io::Error::new(io::ErrorKind::UnexpectedEof, anyhow!("channel closed"))
            })?);
        }
        self.buffer.read(buf)
    }
}

impl AsyncRead for ByteReader {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        // Calculate the amount of remaining bytes in the buffer.
        let mut remaining = self.buffer.get_ref().len() - self.buffer.position() as usize;

        if remaining == 0 {
            let buffer = match self.receiver.poll_recv(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(Ok(())),
                Poll::Ready(Some(v)) => v,
            };
            remaining = buffer.len();
            self.as_mut().buffer = Cursor::new(buffer);
        }

        // Read enough to either fill the buffer we are writing to or exhaust our own buffer.
        let read_amount = buf.remaining().min(remaining);
        buf.put_slice(&self.buffer.get_ref()[..read_amount]);
        // Advance the position in the cursor.
        let prev_pos = self.buffer.position();
        self.buffer.set_position(prev_pos + read_amount as u64);
        Poll::Ready(Ok(()))
    }
}
