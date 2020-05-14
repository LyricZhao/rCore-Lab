use alloc::sync::Arc;
use rcore_fs::vfs::INode;
use rcore_fs_sfs::INodeImpl;
use crate::fs::ROOT_INODE;
use spin::Mutex;
use lazy_static::lazy_static;
use alloc::collections::VecDeque;
use crate::sync::condvar::Condvar;

#[derive(Copy,Clone,Debug)]
pub enum FileDescriptorType {
    FD_NONE,
    FD_INODE,
    FD_DEVICE,
    FD_PIPE,
}

#[derive(Clone)]
pub struct File {
    fdtype: FileDescriptorType,
    readable: bool,
    writable: bool,
    pub inode: Option<Arc<dyn INode>>,
    offset: usize,
    pushed: Arc<Condvar>,
    pipe: Arc<Mutex<VecDeque<u8>>>,
}

impl File {
    pub fn default() -> Self {
        File {
            fdtype: FileDescriptorType::FD_NONE,
            readable: false,
            writable: false,
            inode: None,
            offset: 0,
            pushed: Arc::new(Condvar::new()),
            pipe: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn open_pipe(&mut self, readable: bool, arc: Option<(Arc<Condvar>, Arc<Mutex<VecDeque<u8>>>)>) -> Option<(Arc<Condvar>, Arc<Mutex<VecDeque<u8>>>)> {
        self.set_fdtype(FileDescriptorType::FD_PIPE);
        self.set_readable(readable);
        self.set_writable(!readable);
        if readable {
            Some((self.pushed.clone(), self.pipe.clone()))
        } else {
            if let Some((c, p)) = arc {
                self.pushed = c;
                self.pipe = p;
            } else {
                panic!("something wrong");
            }
            None
        }
    }

    pub fn set_readable(&mut self, v: bool) { self.readable = v; }
    pub fn set_writable(&mut self, v: bool) { self.writable = v; }
    pub fn get_readable(&self) -> bool { self.readable }
    pub fn get_writable(&self) -> bool { self.writable }
    pub fn set_fdtype(&mut self, t: FileDescriptorType) { self.fdtype = t; }
    pub fn get_fdtype(&self) -> FileDescriptorType { self.fdtype }
    pub fn set_offset(&mut self, o: usize) { self.offset = o; }
    pub fn get_offset(&self) -> usize { self.offset }

    pub fn pipe_read(&self) -> u8 {
        loop {
            let option = self.pipe.lock().pop_front();
            match option {
                Some(ch) => {
                    return ch;
                },
                None => {
                    self.pushed.wait();
                }
            }
        }
        unreachable!();
    }

    pub fn pipe_write(&self, ch: u8) {
        self.pipe.lock().push_back(ch);
        self.pushed.notify();
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
