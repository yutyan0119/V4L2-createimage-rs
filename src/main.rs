use bind_v4l2 as v4l2;
mod vidioc;

use libc::{open, O_NONBLOCK, O_RDWR};
use std::ffi::CString;
use std::io::{self, Write};
use std::os::unix::io::RawFd;

struct Device {
    fd: RawFd,
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}

struct Buffer {
    start: *mut std::ffi::c_void,
    length: u32,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { libc::munmap(self.start, self.length as usize) };
    }
}

fn open_device(path: &CString) -> io::Result<Device> {
    let fd: RawFd = unsafe { open(path.as_ptr(), O_RDWR | O_NONBLOCK) };
    if fd == -1 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Failed to open video device: {}",
                io::Error::last_os_error()
            ),
        ));
    }
    let device: Device = Device { fd };
    Ok(device)
}

fn query_device_capabilities(fd: RawFd) -> io::Result<v4l2::v4l2_capability> {
    let mut v4l2_cap: v4l2::v4l2_capability = unsafe { std::mem::zeroed() };
    let ret: std::os::raw::c_int = unsafe {
        libc::ioctl(
            fd,
            vidioc::VIDIOC_QUERYCAP,
            &mut v4l2_cap as *mut _ as *mut std::os::raw::c_void,
        )
    };
    if ret == -1 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Failed to query device capabilities: {}",
                io::Error::last_os_error()
            ),
        ));
    }
    if v4l2_cap.capabilities & vidioc::V4L2_CAP_VIDEO_CAPTURE == 0 {
        println!("Error: V4L2_CAP_VIDEO_CAPTURE is not supported");
    }
    if v4l2_cap.capabilities & vidioc::V4L2_CAP_STREAMING == 0 {
        println!("Error: V4L2_CAP_STREAMING is not supported");
    }
    Ok(v4l2_cap)
}

fn set_capture_format(fd: RawFd, format: &mut v4l2::v4l2_format) -> io::Result<()> {
    let ret = unsafe {
        libc::ioctl(
            fd,
            vidioc::VIDIOC_S_FMT,
            format as *mut _ as *mut std::os::raw::c_void,
        )
    };
    if ret == -1 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Failed to set capture format: {}",
                io::Error::last_os_error()
            ),
        ));
    }
    *format = unsafe { std::mem::zeroed() };
    format.type_ = v4l2::v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
    let ret: std::os::raw::c_int = unsafe {
        libc::ioctl(
            fd,
            vidioc::VIDIOC_G_FMT,
            format as *mut _ as *mut std::os::raw::c_void,
        )
    };
    if ret == -1 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Failed to get capture format: {}",
                io::Error::last_os_error()
            ),
        ));
    }
    Ok(())
}

fn request_buffers(fd: RawFd, count: u32) -> io::Result<v4l2::v4l2_requestbuffers> {
    let mut v4l2_reqbuf: v4l2::v4l2_requestbuffers = unsafe { std::mem::zeroed() };
    v4l2_reqbuf.count = count;
    v4l2_reqbuf.type_ = v4l2::v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
    v4l2_reqbuf.memory = v4l2::v4l2_memory_V4L2_MEMORY_MMAP;
    let ret: std::os::raw::c_int = unsafe {
        libc::ioctl(
            fd,
            vidioc::VIDIOC_REQBUFS,
            &mut v4l2_reqbuf as *mut _ as *mut std::os::raw::c_void,
        )
    };
    if ret == -1 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to request buffers: {}", io::Error::last_os_error()),
        ));
    }
    Ok(v4l2_reqbuf)
}

fn map_buffers(fd: RawFd, v4l2_reqbuf: &v4l2::v4l2_requestbuffers) -> io::Result<Vec<Buffer>> {
    let mut buffers: Vec<Buffer> = Vec::new();
    for index in 0..v4l2_reqbuf.count {
        let mut v4l2_buf: v4l2::v4l2_buffer = unsafe { std::mem::zeroed() };
        v4l2_buf.index = index;
        v4l2_buf.type_ = v4l2::v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
        v4l2_buf.memory = v4l2::v4l2_memory_V4L2_MEMORY_MMAP;
        let ret: std::os::raw::c_int = unsafe {
            libc::ioctl(
                fd,
                vidioc::VIDIOC_QUERYBUF,
                &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
            )
        };
        if ret == -1 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Failed to open video device: {}",
                    io::Error::last_os_error()
                ),
            ));
        }
        let ptr: *mut std::os::raw::c_void = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                v4l2_buf.length as usize,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                v4l2_buf.m.offset as libc::off_t,
            )
        };
        if ptr == libc::MAP_FAILED {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Failed to open video device: {}",
                    io::Error::last_os_error()
                ),
            ));
        }
        let buffer: Buffer = Buffer {
            start: ptr,
            length: v4l2_buf.length,
        };
        buffers.push(buffer);
    }

    for index in 0..v4l2_reqbuf.count {
        let mut v4l2_buf: v4l2::v4l2_buffer = unsafe { std::mem::zeroed() };
        v4l2_buf.index = index;
        v4l2_buf.type_ = v4l2::v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
        v4l2_buf.memory = v4l2::v4l2_memory_V4L2_MEMORY_MMAP;
        let ret: std::os::raw::c_int = unsafe {
            libc::ioctl(
                fd,
                vidioc::VIDIOC_QBUF,
                &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
            )
        };
        if ret == -1 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Failed to open video device: {}",
                    io::Error::last_os_error()
                ),
            ));
        }
    }

    Ok(buffers)
}

fn start_streaming(fd: RawFd) -> io::Result<()> {
    let mut v4l2_buf_type: v4l2::v4l2_buf_type = v4l2::v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
    let ret: std::os::raw::c_int = unsafe {
        libc::ioctl(
            fd,
            vidioc::VIDIOC_STREAMON,
            &mut v4l2_buf_type as *mut _ as *mut std::os::raw::c_void,
        )
    };

    if ret == -1 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Failed to open video device: {}",
                io::Error::last_os_error()
            ),
        ));
    }
    Ok(())
}

fn stop_streaming(fd: RawFd) -> io::Result<()> {
    let mut v4l2_buf_type: v4l2::v4l2_buf_type = v4l2::v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
    let ret: std::os::raw::c_int = unsafe {
        libc::ioctl(
            fd,
            vidioc::VIDIOC_STREAMOFF,
            &mut v4l2_buf_type as *mut _ as *mut std::os::raw::c_void,
        )
    };

    if ret == -1 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Failed to open video device: {}",
                io::Error::last_os_error()
            ),
        ));
    }
    Ok(())
}

fn capture_frame(fd: RawFd, buffers: &[Buffer]) -> io::Result<&[u8]> {
    let mut v4l2_buf: v4l2::v4l2_buffer = unsafe { std::mem::zeroed() };
    v4l2_buf.type_ = v4l2::v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
    v4l2_buf.memory = v4l2::v4l2_memory_V4L2_MEMORY_MMAP;
    let ret: i32 = unsafe {
        libc::poll(
            &mut libc::pollfd {
                fd: fd,
                events: libc::POLLIN,
                revents: 0,
            },
            1,
            1000,
        )
    };
    if ret == -1 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Failed to open video device: {}",
                io::Error::last_os_error()
            ),
        ));
    }
    let ret: std::os::raw::c_int = unsafe {
        libc::ioctl(
            fd,
            vidioc::VIDIOC_DQBUF,
            &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
        )
    };

    if ret == -1 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Failed to open video device: {}",
                io::Error::last_os_error()
            ),
        ));
    }

    let data_slice: &[u8] = unsafe {
        std::slice::from_raw_parts::<u8>(
            buffers[v4l2_buf.index as usize].start as *mut u8,
            buffers[v4l2_buf.index as usize].length as usize,
        )
    };
    Ok(data_slice)
}

fn save_frame_to_file(data: &[u8], filename: &str) -> io::Result<()> {
    let mut file: std::fs::File = std::fs::File::create(filename)?;
    file.write_all(data)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let path = CString::new("/dev/video0")?;
    let bufcount = 60;
    let mut format: v4l2::v4l2_format = unsafe { std::mem::zeroed() };
    format.type_ = v4l2::v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
    format.fmt.pix.width = 1280;
    format.fmt.pix.height = 720;
    format.fmt.pix.pixelformat = vidioc::v4l2_fourcc(b'M', b'J', b'P', b'G');
    format.fmt.pix.field = v4l2::v4l2_field_V4L2_FIELD_ANY;
    let device: Device = open_device(&path)?;

    let _ = query_device_capabilities(device.fd)?;

    let _ = set_capture_format(device.fd, &mut format)?;

    unsafe {
        println!("width: {}", format.fmt.pix.width);
        println!("height: {}", format.fmt.pix.height);
        println!("pixelformat: {}", format.fmt.pix.pixelformat);
    }

    let v4l2_reqbuf: v4l2::v4l2_requestbuffers = request_buffers(device.fd, bufcount)?;

    println!("buffers: {}", v4l2_reqbuf.count);

    let buffers: Vec<Buffer> = map_buffers(device.fd, &v4l2_reqbuf)?;

    start_streaming(device.fd)?;

    let data: &[u8] = capture_frame(device.fd, &buffers)?;

    save_frame_to_file(&data, "test.jpg")?;

    stop_streaming(device.fd)?;
    Ok(())
}
