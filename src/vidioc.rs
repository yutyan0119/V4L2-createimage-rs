use bind_v4l2::*;

#[allow(non_camel_case_types)]
pub type _IOC_TYPE = std::os::raw::c_ulong;

// linux ioctl.h
const _IOC_NRBITS: u8 = 8;
const _IOC_TYPEBITS: u8 = 8;

const _IOC_SIZEBITS: u8 = 14;

const _IOC_NRSHIFT: u8 = 0;
const _IOC_TYPESHIFT: u8 = _IOC_NRSHIFT + _IOC_NRBITS;
const _IOC_SIZESHIFT: u8 = _IOC_TYPESHIFT + _IOC_TYPEBITS;
const _IOC_DIRSHIFT: u8 = _IOC_SIZESHIFT + _IOC_SIZEBITS;

const _IOC_NONE: u8 = 0;
const _IOC_WRITE: u8 = 1;
const _IOC_READ: u8 = 2;

macro_rules! _IOC_TYPECHECK {
    ($type:ty) => {
        std::mem::size_of::<$type>()
    };
}

macro_rules! _IOC {
    ($dir:expr, $type:expr, $nr:expr, $size:expr) => {
        (($dir as _IOC_TYPE) << $crate::vidioc::_IOC_DIRSHIFT)
            | (($type as _IOC_TYPE) << $crate::vidioc::_IOC_TYPESHIFT)
            | (($nr as _IOC_TYPE) << $crate::vidioc::_IOC_NRSHIFT)
            | (($size as _IOC_TYPE) << $crate::vidioc::_IOC_SIZESHIFT)
    };
}

macro_rules! _IO {
    ($type:expr, $nr:expr) => {
        _IOC!($crate::vidioc::_IOC_NONE, $type, $nr, 0)
    };
}

macro_rules! _IOR {
    ($type:expr, $nr:expr, $size:ty) => {
        _IOC!(
            $crate::vidioc::_IOC_READ,
            $type,
            $nr,
            _IOC_TYPECHECK!($size)
        )
    };
}

macro_rules! _IOW {
    ($type:expr, $nr:expr, $size:ty) => {
        _IOC!(
            $crate::vidioc::_IOC_WRITE,
            $type,
            $nr,
            _IOC_TYPECHECK!($size)
        )
    };
}

macro_rules! _IOWR {
    ($type:expr, $nr:expr, $size:ty) => {
        _IOC!(
            $crate::vidioc::_IOC_READ | $crate::vidioc::_IOC_WRITE,
            $type,
            $nr,
            _IOC_TYPECHECK!($size)
        )
    };
}

pub const VIDIOC_QUERYCAP: _IOC_TYPE = _IOR!(b'V', 0, v4l2_capability);
pub const VIDIOC_G_FMT: _IOC_TYPE = _IOWR!(b'V', 4, v4l2_format);
pub const VIDIOC_S_FMT: _IOC_TYPE = _IOWR!(b'V', 5, v4l2_format);
pub const VIDIOC_REQBUFS: _IOC_TYPE = _IOWR!(b'V', 8, v4l2_requestbuffers);
pub const VIDIOC_QUERYBUF: _IOC_TYPE = _IOWR!(b'V', 9, v4l2_buffer);
pub const VIDIOC_QBUF: _IOC_TYPE = _IOWR!(b'V', 15, v4l2_buffer);
pub const VIDIOC_DQBUF: _IOC_TYPE = _IOWR!(b'V', 17, v4l2_buffer);
pub const VIDIOC_STREAMON: _IOC_TYPE = _IOW!(b'V', 18, std::os::raw::c_int);
pub const VIDIOC_STREAMOFF: _IOC_TYPE = _IOW!(b'V', 19, std::os::raw::c_int);

pub const V4L2_CAP_VIDEO_CAPTURE: u32 = 0x00000001; /* Is a video capture device */
pub const V4L2_CAP_STREAMING: u32 = 0x04000000; /* Streaming I/O ioctls */

pub fn v4l2_fourcc(a: u8, b: u8, c: u8, d: u8) -> u32 {
    (a as u32) | ((b as u32) << 8) | ((c as u32) << 16) | ((d as u32) << 24)
}
