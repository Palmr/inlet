use crate::array_string::ArrayString;
use libc::{MAP_FILE, MAP_SHARED_VALIDATE, MAP_SYNC, PROT_READ, PROT_WRITE};
use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};
use std::mem;
use std::mem::size_of;
use std::os::fd::AsRawFd;
use std::ptr::{addr_of, null_mut, slice_from_raw_parts};

const PAD_SIZE: usize = 64;

#[repr(C)]
pub struct Meta {
    pub topic: ArrayString,
    pub type_size: usize,
    pub entry_count: usize,
    pub max_consumers: usize,
    pub initialised: bool,
    alignment_pad: [u8; 7],
}

#[repr(C)]
pub struct Inlet<EntryType, const ENTRY_COUNT: usize, const MAX_CONSUMERS: usize> {
    pub meta: Meta,
    pad1: [u8; PAD_SIZE],
    pub data: [EntryType; ENTRY_COUNT],
    pub producer: ClientMeta,
    pub consumers: [ClientMeta; MAX_CONSUMERS],
    pad2: [u8; PAD_SIZE],
}

#[repr(C)]
pub struct ClientMeta {
    pad: [u8; PAD_SIZE],
    pub id: ArrayString,
    pub sequence: usize,
    pub timestamp: u64,
}

impl ClientMeta {
    const EMPTY: ClientMeta = ClientMeta {
        pad: [0; PAD_SIZE],
        id: ArrayString::empty(),
        sequence: 0,
        timestamp: 0,
    };
}

impl<EntryType, const ENTRY_COUNT: usize, const MAX_CONSUMERS: usize>
    Inlet<EntryType, ENTRY_COUNT, MAX_CONSUMERS>
{
    pub fn construct<I: Into<ArrayString>>(topic: I) -> *mut Self {
        let topic = topic.into();
        let filename = format!("inlet-{}", topic.to_string());
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&filename);

        let size = size_of::<Inlet<EntryType, ENTRY_COUNT, MAX_CONSUMERS>>();

        match file {
            Ok(mut f) => {
                //fill file
                let wormhole = Self {
                    meta: Meta {
                        topic,
                        type_size: size_of::<EntryType>(),
                        entry_count: ENTRY_COUNT,
                        max_consumers: MAX_CONSUMERS,
                        initialised: false,
                        alignment_pad: [0; 7],
                    },
                    pad1: [0; PAD_SIZE],
                    data: unsafe { mem::zeroed() },
                    producer: ClientMeta {
                        pad: [0; PAD_SIZE],
                        id: ArrayString::empty(),
                        sequence: 0,
                        timestamp: 0,
                    },
                    consumers: [ClientMeta::EMPTY; MAX_CONSUMERS],
                    pad2: [0; PAD_SIZE],
                };

                let slice = unsafe {
                    slice_from_raw_parts(addr_of!(wormhole).cast::<u8>(), size)
                        .as_ref()
                        .unwrap()
                };
                f.write_all(slice).unwrap();
                let inlet_ptr = unsafe {
                    libc::mmap64(
                        null_mut(),
                        size,
                        PROT_WRITE | PROT_READ,
                        MAP_SHARED_VALIDATE | MAP_FILE,
                        f.as_raw_fd(),
                        0,
                    )
                    .cast::<Inlet<EntryType, ENTRY_COUNT, MAX_CONSUMERS>>()
                };

                unsafe { &mut *inlet_ptr }.meta.initialised = true;

                unsafe {
                    libc::msync(inlet_ptr.cast(), 1000, MAP_SYNC);
                }

                inlet_ptr
            }
            Err(e) => {
                if e.kind() == ErrorKind::AlreadyExists {
                    let file = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(&filename)
                        .expect("Could not open file");
                    unsafe {
                        libc::mmap64(
                            null_mut(),
                            size,
                            PROT_WRITE | PROT_READ,
                            MAP_SHARED_VALIDATE | MAP_FILE,
                            file.as_raw_fd(),
                            0,
                        )
                        .cast::<Inlet<EntryType, ENTRY_COUNT, MAX_CONSUMERS>>()
                    }

                    // wait in here until initialised true?
                } else {
                    panic!("argh");
                }
            }
        }
    }
}

#[cfg(test)]
mod wormhole_tests {
    use crate::inlet::Inlet;
    use std::fs;

    struct TestEntry {}

    fn cleanup(topic: &str) {
        fs::remove_file(format!("inlet-{}", topic)).ok();
    }

    #[test]
    fn test_initialise_wormhole() {
        cleanup("test");
        let ih = Inlet::<TestEntry, 8, 2>::construct(String::from("test"));
        let inlet = unsafe { &mut *ih };
        println!("{:?}", inlet.meta.topic);
    }
}
