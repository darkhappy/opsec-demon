//! Timeout-bounded overlapped I/O for the local Discord Windows named pipe.
use std::ffi::c_void;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::AsRawHandle;
use std::ptr::null_mut;

type Handle = *mut c_void;
#[repr(C)]
#[derive(Default)]
struct Overlapped {
    internal: usize,
    internal_high: usize,
    offset: u32,
    offset_high: u32,
    event: Handle,
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateEventW(attributes: Handle, manual: i32, initial: i32, name: *const u16) -> Handle;
    fn CloseHandle(handle: Handle) -> i32;
    fn ReadFile(
        file: Handle,
        buffer: Handle,
        length: u32,
        count: *mut u32,
        overlap: *mut Overlapped,
    ) -> i32;
    fn WriteFile(
        file: Handle,
        buffer: Handle,
        length: u32,
        count: *mut u32,
        overlap: *mut Overlapped,
    ) -> i32;
    fn WaitForSingleObject(handle: Handle, timeout: u32) -> u32;
    fn GetOverlappedResult(
        file: Handle,
        overlap: *mut Overlapped,
        count: *mut u32,
        wait: i32,
    ) -> i32;
    fn CancelIoEx(file: Handle, overlap: *mut Overlapped) -> i32;
}

pub struct Pipe(File);
impl Pipe {
    pub fn open(index: usize) -> io::Result<Self> {
        Self::open_path(&format!(r"\\.\pipe\discord-ipc-{index}"))
    }

    fn open_path(path: &str) -> io::Result<Self> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(0x40000000)
            .open(path)
            .map(Self)
    }

    fn transfer(&self, buffer: Handle, length: usize, write: bool) -> io::Result<usize> {
        // The buffer and OVERLAPPED remain alive until completion, including cancellation.
        unsafe {
            let event = CreateEventW(null_mut(), 1, 0, null_mut());
            if event.is_null() {
                return Err(io::Error::last_os_error());
            }
            let mut overlap = Overlapped {
                event,
                ..Default::default()
            };
            let handle = self.0.as_raw_handle();
            let mut count = 0;
            let length = length.min(u32::MAX as usize) as u32;
            let result = if write {
                WriteFile(handle, buffer, length, &mut count, &mut overlap)
            } else {
                ReadFile(handle, buffer, length, &mut count, &mut overlap)
            };
            let error = io::Error::last_os_error();
            let answer = if result != 0 {
                Ok(count as usize)
            } else if error.raw_os_error() != Some(997) {
                // ERROR_IO_PENDING
                Err(error)
            } else if WaitForSingleObject(event, 500) == 0 {
                if GetOverlappedResult(handle, &mut overlap, &mut count, 0) != 0 {
                    Ok(count as usize)
                } else {
                    Err(io::Error::last_os_error())
                }
            } else {
                CancelIoEx(handle, &mut overlap);
                // Complete cancellation before releasing the stack or borrowed buffer.
                GetOverlappedResult(handle, &mut overlap, &mut count, 1);
                Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Discord IPC timed out",
                ))
            };
            CloseHandle(event);
            answer
        }
    }
}
impl Read for Pipe {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.transfer(buffer.as_mut_ptr().cast(), buffer.len(), false)
    }
}
impl Write for Pipe {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.transfer(buffer.as_ptr().cast_mut().cast(), buffer.len(), true)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::windows::io::FromRawHandle;
    use std::thread;
    use std::time::{Duration, Instant};
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateNamedPipeW(
            name: *const u16,
            access: u32,
            mode: u32,
            instances: u32,
            output: u32,
            input: u32,
            timeout: u32,
            attributes: Handle,
        ) -> Handle;
        fn ConnectNamedPipe(pipe: Handle, overlap: Handle) -> i32;
    }
    #[test]
    fn real_named_pipe_transfers_and_cancels_pending_read() {
        let path = format!(r"\\.\pipe\opsec-demon-test-{}", std::process::id());
        let name: Vec<_> = path.encode_utf16().chain(Some(0)).collect();
        let handle =
            unsafe { CreateNamedPipeW(name.as_ptr(), 3, 0, 1, 4096, 4096, 500, null_mut()) };
        assert_ne!(handle as isize, -1);
        let mut server = unsafe { File::from_raw_handle(handle) };
        let peer = thread::spawn(move || {
            unsafe {
                ConnectNamedPipe(server.as_raw_handle(), null_mut());
            }
            let mut request = [0; 4];
            server.read_exact(&mut request).unwrap();
            assert_eq!(&request, b"PING");
            server.write_all(b"PONG").unwrap();
            thread::sleep(Duration::from_millis(850));
        });
        let mut client = Pipe::open_path(&path).unwrap();
        client.write_all(b"PING").unwrap();
        let mut reply = [0; 4];
        client.read_exact(&mut reply).unwrap();
        assert_eq!(&reply, b"PONG");
        let started = Instant::now();
        assert_eq!(
            client.read(&mut reply).unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
        assert!(started.elapsed() < Duration::from_millis(800));
        drop(client);
        peer.join().unwrap();
    }
}
