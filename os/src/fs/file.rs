use alloc::sync::Arc;
use rcore_fs::vfs::INode;
use rcore_fs_sfs::INodeImpl;
use crate::fs::ROOT_INODE;
use spin::Mutex;
use lazy_static::lazy_static;
use alloc::collections::VecDeque;

#[derive(Copy,Clone,Debug)]
pub enum FileDescriptorType {
    FD_NONE,
    FD_INODE,
    FD_DEVICE,
    FD_PIPE,
}

lazy_static! {
    static ref PIPE: Mutex<VecDeque<u8>> = Mutex::new(VecDeque::default());
}

#[derive(Clone)]
pub struct File {
    fdtype: FileDescriptorType,
    readable: bool,
    writable: bool,
    pub inode: Option<Arc<dyn INode>>,
    offset: usize,
}

impl File {
    pub fn default() -> Self {
        File {
            fdtype: FileDescriptorType::FD_NONE,
            readable: false,
            writable: false,
            inode: None,
            offset: 0,
        }
    }
    pub fn open_pipe(&mut self, readable: bool) {
        self.set_fdtype(FileDescriptorType::FD_PIPE);
        self.set_readable(readable);
        self.set_writable(!readable);
    }

    pub fn set_readable(&mut self, v: bool) { self.readable = v; }
    pub fn set_writable(&mut self, v: bool) { self.writable = v; }
    pub fn get_readable(&self) -> bool { self.readable }
    pub fn get_writable(&self) -> bool { self.writable }
    pub fn set_fdtype(&mut self, t: FileDescriptorType) { self.fdtype = t; }
    pub fn get_fdtype(&self) -> FileDescriptorType { self.fdtype }
    pub fn set_offset(&mut self, o: usize) { self.offset = o; }
    pub fn get_offset(&self) -> usize { self.offset }

    pub fn pipe_read(&self) -> Option<u8> {
        PIPE.lock().pop_front()
    }

    pub fn pipe_write(&self, ch: u8) {
        PIPE.lock().push_back(ch);
    }

    pub fn open_file(&mut self, path: &'static str, flags: i32) {
        self.set_fdtype(FileDescriptorType::FD_INODE);
        self.set_readable(true);
        if (flags & 1) > 0 {
            self.set_readable(false);
        }
        if (flags & 3) > 0 {
            self.set_writable(true);
        }
        unsafe {
            self.inode = Some(ROOT_INODE.lookup(path).unwrap().clone());
        }
        self.set_offset(0);
    }
}
