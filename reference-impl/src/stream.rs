use std::os::unix::io::RawFd;
use std::ptr;

#[derive(Debug)]
pub struct Stream {
    stream: *mut ffi::mmapStream_t,
}

impl Stream {
    #[must_use]
    pub fn new(fd: RawFd) -> Self {
        let stream = unsafe { ffi::streamCreate(fd) };
        Self { stream }
    }

    #[inline]
    pub fn byte_align(&mut self) {
        unsafe { ffi::streamByteAlign(self.stream) }
    }

    #[inline]
    pub fn read_uvar(&mut self) -> u32 {
        self.byte_align();
        unsafe { ffi::streamReadUnsignedVB(self.stream) }
    }

    #[inline]
    pub fn read_ivar(&mut self) -> i32 {
        self.byte_align();
        unsafe { ffi::streamReadSignedVB(self.stream) }
    }

    #[inline]
    pub fn read_u32_elias_delta(&mut self) -> u32 {
        unsafe { ffi::streamReadEliasDeltaU32(self.stream) }
    }

    #[inline]
    pub fn read_i32_elias_delta(&mut self) -> i32 {
        unsafe { ffi::streamReadEliasDeltaS32(self.stream) }
    }

    #[inline]
    pub fn read_tagged_16_v1(&mut self) -> [i64; 4] {
        self.byte_align();

        let mut result = [0; 4];
        let pointer = ptr::addr_of_mut!(result[0]);
        unsafe { ffi::streamReadTag8_4S16_v1(self.stream, pointer) }
        result
    }

    #[inline]
    pub fn read_tagged_16_v2(&mut self) -> [i64; 4] {
        self.byte_align();

        let mut result = [0; 4];
        let pointer = ptr::addr_of_mut!(result[0]);
        unsafe { ffi::streamReadTag8_4S16_v2(self.stream, pointer) }
        result
    }

    #[inline]
    pub fn read_tagged_32(&mut self) -> [i64; 3] {
        self.byte_align();

        let mut result = [0; 3];
        let pointer = ptr::addr_of_mut!(result[0]);
        unsafe { ffi::streamReadTag2_3S32(self.stream, pointer) }
        result
    }

    #[inline]
    pub fn read_bits(&mut self, bits: u8) -> u32 {
        unsafe { ffi::streamReadBits(self.stream, bits as i32) }
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        unsafe { ffi::streamDestroy(self.stream) };
    }
}

mod ffi {
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    // TODO: switch to std::ffi
    use std::os::raw::{c_char, c_int, c_long, c_uint, c_ulong};

    type __dev_t = c_ulong;
    type __uid_t = c_uint;
    type __gid_t = c_uint;
    type __ino_t = c_ulong;
    type __mode_t = c_uint;
    type __nlink_t = c_ulong;
    type __off_t = c_long;
    type __time_t = c_long;
    type __blksize_t = c_long;
    type __blkcnt_t = c_long;
    type __syscall_slong_t = c_long;

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    struct timespec {
        tv_sec: __time_t,
        tv_nsec: __syscall_slong_t,
    }

    #[test]
    fn bindgen_test_layout_timespec() {
        use std::mem;
        use std::mem::MaybeUninit;
        use std::ptr;

        assert_eq!(
            mem::size_of::<timespec>(),
            16usize,
            concat!("Size of: ", stringify!(timespec))
        );
        assert_eq!(
            mem::align_of::<timespec>(),
            8usize,
            concat!("Alignment of ", stringify!(timespec))
        );
        fn test_field_tv_sec() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<timespec>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).tv_sec) as usize - ptr as usize
                },
                0usize,
                concat!(
                    "Offset of field: ",
                    stringify!(timespec),
                    "::",
                    stringify!(tv_sec)
                )
            );
        }
        test_field_tv_sec();
        fn test_field_tv_nsec() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<timespec>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).tv_nsec) as usize - ptr as usize
                },
                8usize,
                concat!(
                    "Offset of field: ",
                    stringify!(timespec),
                    "::",
                    stringify!(tv_nsec)
                )
            );
        }
        test_field_tv_nsec();
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    struct stat {
        st_dev: __dev_t,
        st_ino: __ino_t,
        st_nlink: __nlink_t,
        st_mode: __mode_t,
        st_uid: __uid_t,
        st_gid: __gid_t,
        __pad0: c_int,
        st_rdev: __dev_t,
        st_size: __off_t,
        st_blksize: __blksize_t,
        st_blocks: __blkcnt_t,
        st_atim: timespec,
        st_mtim: timespec,
        st_ctim: timespec,
        __glibc_reserved: [__syscall_slong_t; 3usize],
    }

    #[test]
    fn bindgen_test_layout_stat() {
        use std::mem;
        use std::mem::MaybeUninit;
        use std::ptr;

        assert_eq!(
            mem::size_of::<stat>(),
            144usize,
            concat!("Size of: ", stringify!(stat))
        );
        assert_eq!(
            mem::align_of::<stat>(),
            8usize,
            concat!("Alignment of ", stringify!(stat))
        );
        fn test_field_st_dev() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<stat>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).st_dev) as usize - ptr as usize
                },
                0usize,
                concat!(
                    "Offset of field: ",
                    stringify!(stat),
                    "::",
                    stringify!(st_dev)
                )
            );
        }
        test_field_st_dev();
        fn test_field_st_ino() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<stat>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).st_ino) as usize - ptr as usize
                },
                8usize,
                concat!(
                    "Offset of field: ",
                    stringify!(stat),
                    "::",
                    stringify!(st_ino)
                )
            );
        }
        test_field_st_ino();
        fn test_field_st_nlink() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<stat>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).st_nlink) as usize - ptr as usize
                },
                16usize,
                concat!(
                    "Offset of field: ",
                    stringify!(stat),
                    "::",
                    stringify!(st_nlink)
                )
            );
        }
        test_field_st_nlink();
        fn test_field_st_mode() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<stat>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).st_mode) as usize - ptr as usize
                },
                24usize,
                concat!(
                    "Offset of field: ",
                    stringify!(stat),
                    "::",
                    stringify!(st_mode)
                )
            );
        }
        test_field_st_mode();
        fn test_field_st_uid() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<stat>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).st_uid) as usize - ptr as usize
                },
                28usize,
                concat!(
                    "Offset of field: ",
                    stringify!(stat),
                    "::",
                    stringify!(st_uid)
                )
            );
        }
        test_field_st_uid();
        fn test_field_st_gid() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<stat>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).st_gid) as usize - ptr as usize
                },
                32usize,
                concat!(
                    "Offset of field: ",
                    stringify!(stat),
                    "::",
                    stringify!(st_gid)
                )
            );
        }
        test_field_st_gid();
        fn test_field___pad0() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<stat>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).__pad0) as usize - ptr as usize
                },
                36usize,
                concat!(
                    "Offset of field: ",
                    stringify!(stat),
                    "::",
                    stringify!(__pad0)
                )
            );
        }
        test_field___pad0();
        fn test_field_st_rdev() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<stat>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).st_rdev) as usize - ptr as usize
                },
                40usize,
                concat!(
                    "Offset of field: ",
                    stringify!(stat),
                    "::",
                    stringify!(st_rdev)
                )
            );
        }
        test_field_st_rdev();
        fn test_field_st_size() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<stat>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).st_size) as usize - ptr as usize
                },
                48usize,
                concat!(
                    "Offset of field: ",
                    stringify!(stat),
                    "::",
                    stringify!(st_size)
                )
            );
        }
        test_field_st_size();
        fn test_field_st_blksize() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<stat>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).st_blksize) as usize - ptr as usize
                },
                56usize,
                concat!(
                    "Offset of field: ",
                    stringify!(stat),
                    "::",
                    stringify!(st_blksize)
                )
            );
        }
        test_field_st_blksize();
        fn test_field_st_blocks() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<stat>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).st_blocks) as usize - ptr as usize
                },
                64usize,
                concat!(
                    "Offset of field: ",
                    stringify!(stat),
                    "::",
                    stringify!(st_blocks)
                )
            );
        }
        test_field_st_blocks();
        fn test_field_st_atim() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<stat>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).st_atim) as usize - ptr as usize
                },
                72usize,
                concat!(
                    "Offset of field: ",
                    stringify!(stat),
                    "::",
                    stringify!(st_atim)
                )
            );
        }
        test_field_st_atim();
        fn test_field_st_mtim() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<stat>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).st_mtim) as usize - ptr as usize
                },
                88usize,
                concat!(
                    "Offset of field: ",
                    stringify!(stat),
                    "::",
                    stringify!(st_mtim)
                )
            );
        }
        test_field_st_mtim();
        fn test_field_st_ctim() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<stat>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).st_ctim) as usize - ptr as usize
                },
                104usize,
                concat!(
                    "Offset of field: ",
                    stringify!(stat),
                    "::",
                    stringify!(st_ctim)
                )
            );
        }
        test_field_st_ctim();
        fn test_field___glibc_reserved() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<stat>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).__glibc_reserved) as usize - ptr as usize
                },
                120usize,
                concat!(
                    "Offset of field: ",
                    stringify!(stat),
                    "::",
                    stringify!(__glibc_reserved)
                )
            );
        }
        test_field___glibc_reserved();
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    struct fileMapping_t {
        fd: c_int,
        data: *mut c_char,
        stats: stat,
        size: usize,
    }

    #[test]
    fn bindgen_test_layout_fileMapping_t() {
        use std::mem;
        use std::mem::MaybeUninit;
        use std::ptr;

        assert_eq!(
            mem::size_of::<fileMapping_t>(),
            168usize,
            concat!("Size of: ", stringify!(fileMapping_t))
        );
        assert_eq!(
            mem::align_of::<fileMapping_t>(),
            8usize,
            concat!("Alignment of ", stringify!(fileMapping_t))
        );
        fn test_field_fd() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<fileMapping_t>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).fd) as usize - ptr as usize
                },
                0usize,
                concat!(
                    "Offset of field: ",
                    stringify!(fileMapping_t),
                    "::",
                    stringify!(fd)
                )
            );
        }
        test_field_fd();
        fn test_field_data() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<fileMapping_t>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).data) as usize - ptr as usize
                },
                8usize,
                concat!(
                    "Offset of field: ",
                    stringify!(fileMapping_t),
                    "::",
                    stringify!(data)
                )
            );
        }
        test_field_data();
        fn test_field_stats() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<fileMapping_t>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).stats) as usize - ptr as usize
                },
                16usize,
                concat!(
                    "Offset of field: ",
                    stringify!(fileMapping_t),
                    "::",
                    stringify!(stats)
                )
            );
        }
        test_field_stats();
        fn test_field_size() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<fileMapping_t>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).size) as usize - ptr as usize
                },
                160usize,
                concat!(
                    "Offset of field: ",
                    stringify!(fileMapping_t),
                    "::",
                    stringify!(size)
                )
            );
        }
        test_field_size();
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub(super) struct mmapStream_t {
        mapping: fileMapping_t,
        data: *const c_char,
        size: usize,
        start: *const c_char,
        end: *const c_char,
        pos: *const c_char,
        bitPos: c_int,
        eof: bool,
    }

    #[test]
    fn bindgen_test_layout_mmapStream_t() {
        use std::mem;
        use std::mem::MaybeUninit;
        use std::ptr;

        assert_eq!(
            mem::size_of::<mmapStream_t>(),
            216usize,
            concat!("Size of: ", stringify!(mmapStream_t))
        );
        assert_eq!(
            mem::align_of::<mmapStream_t>(),
            8usize,
            concat!("Alignment of ", stringify!(mmapStream_t))
        );
        fn test_field_mapping() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<mmapStream_t>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).mapping) as usize - ptr as usize
                },
                0usize,
                concat!(
                    "Offset of field: ",
                    stringify!(mmapStream_t),
                    "::",
                    stringify!(mapping)
                )
            );
        }
        test_field_mapping();
        fn test_field_data() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<mmapStream_t>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).data) as usize - ptr as usize
                },
                168usize,
                concat!(
                    "Offset of field: ",
                    stringify!(mmapStream_t),
                    "::",
                    stringify!(data)
                )
            );
        }
        test_field_data();
        fn test_field_size() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<mmapStream_t>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).size) as usize - ptr as usize
                },
                176usize,
                concat!(
                    "Offset of field: ",
                    stringify!(mmapStream_t),
                    "::",
                    stringify!(size)
                )
            );
        }
        test_field_size();
        fn test_field_start() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<mmapStream_t>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).start) as usize - ptr as usize
                },
                184usize,
                concat!(
                    "Offset of field: ",
                    stringify!(mmapStream_t),
                    "::",
                    stringify!(start)
                )
            );
        }
        test_field_start();
        fn test_field_end() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<mmapStream_t>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).end) as usize - ptr as usize
                },
                192usize,
                concat!(
                    "Offset of field: ",
                    stringify!(mmapStream_t),
                    "::",
                    stringify!(end)
                )
            );
        }
        test_field_end();
        fn test_field_pos() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<mmapStream_t>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).pos) as usize - ptr as usize
                },
                200usize,
                concat!(
                    "Offset of field: ",
                    stringify!(mmapStream_t),
                    "::",
                    stringify!(pos)
                )
            );
        }
        test_field_pos();
        fn test_field_bitPos() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<mmapStream_t>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).bitPos) as usize - ptr as usize
                },
                208usize,
                concat!(
                    "Offset of field: ",
                    stringify!(mmapStream_t),
                    "::",
                    stringify!(bitPos)
                )
            );
        }
        test_field_bitPos();
        fn test_field_eof() {
            assert_eq!(
                unsafe {
                    let uninit = MaybeUninit::<mmapStream_t>::uninit();
                    let ptr = uninit.as_ptr();
                    ptr::addr_of!((*ptr).eof) as usize - ptr as usize
                },
                212usize,
                concat!(
                    "Offset of field: ",
                    stringify!(mmapStream_t),
                    "::",
                    stringify!(eof)
                )
            );
        }
        test_field_eof();
    }

    extern "C" {
        pub(super) fn streamCreate(fd: c_int) -> *mut mmapStream_t;
        pub(super) fn streamDestroy(stream: *mut mmapStream_t);

        // fn streamPeekChar(stream: *mut mmapStream_t) -> c_int;
        // fn streamReadChar(stream: *mut mmapStream_t) -> c_char;
        // fn streamReadByte(stream: *mut mmapStream_t) -> c_int;
        // fn streamUnreadChar(stream: *mut mmapStream_t);
        // fn streamRead(stream: *mut mmapStream_t, buf: *mut c_void, len: c_int);

        pub(super) fn streamReadBits(stream: *mut mmapStream_t, numBits: c_int) -> u32;

        // fn streamReadBit(stream: *mut mmapStream_t) -> c_int;

        pub(super) fn streamByteAlign(stream: *mut mmapStream_t);

        // fn streamReadS16(stream: *mut mmapStream_t) -> i16;
        // fn streamReadRawFloat(stream: *mut mmapStream_t) -> f32;

        pub(super) fn streamReadUnsignedVB(stream: *mut mmapStream_t) -> u32;
        pub(super) fn streamReadSignedVB(stream: *mut mmapStream_t) -> i32;

        pub(super) fn streamReadTag2_3S32(stream: *mut mmapStream_t, values: *mut i64);
        pub(super) fn streamReadTag8_4S16_v1(stream: *mut mmapStream_t, values: *mut i64);
        pub(super) fn streamReadTag8_4S16_v2(stream: *mut mmapStream_t, values: *mut i64);

        // pub(super) fn streamReadTag8_8SVB(
        //     stream: *mut mmapStream_t,
        //     values: *mut i64,
        //     valueCount: c_int,
        // );

        pub(super) fn streamReadEliasDeltaU32(stream: *mut mmapStream_t) -> u32;
        pub(super) fn streamReadEliasDeltaS32(stream: *mut mmapStream_t) -> i32;
        // pub(super) fn streamReadEliasGammaU32(stream: *mut mmapStream_t) -> u32;
        // pub(super) fn streamReadEliasGammaS32(stream: *mut mmapStream_t) -> i32;
    }
}
