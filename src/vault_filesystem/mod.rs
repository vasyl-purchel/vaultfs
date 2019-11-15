extern crate fuse;
extern crate libc;
extern crate env_logger;
extern crate time;
extern crate clap;
extern crate reqwest;
extern crate serde_json;

use std::ffi::OsStr;
use fuse::{FileType, FileAttr, Filesystem, Request, ReplyData, ReplyEntry, ReplyAttr, ReplyDirectory};
use libc::ENOENT;
use time::Timespec;

pub mod vault_fs_tree;

pub struct VaultFilesystem {
  fs_tree: vault_fs_tree::VaultFsTree,
}

impl VaultFilesystem {
  pub fn new(tree: vault_fs_tree::VaultFsTree) -> VaultFilesystem {
    return VaultFilesystem { fs_tree: tree };
  }

  fn dir_attr(&self, ino: u64) -> FileAttr {
      let ttl = Timespec::new(2, 0);
      return FileAttr {
          ino: ino,
          size: 0,
          blocks: 0,
          atime: ttl,
          mtime: ttl,
          ctime: ttl,
          crtime: ttl,
          kind: FileType::Directory,
          perm: 0o755,
          nlink: 2,
          uid: 0,
          gid: 0,
          rdev: 0,
          flags: 0,
      };
  }

  fn file_attr(&self, ino: u64) -> FileAttr {
      let ttl = Timespec::new(2, 0);
      return FileAttr {
          ino: ino,
          size: 13,
          blocks: 1,
          atime: ttl,
          mtime: ttl,
          ctime: ttl,
          crtime: ttl,
          kind: FileType::RegularFile,
          perm: 0o644,
          nlink: 1,
          uid: 0,
          gid: 0,
          rdev: 0,
          flags: 0,
      };
  }
}

impl Filesystem for VaultFilesystem {
  fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
      println!("getattr(ino={})", ino);
      let ttl = Timespec::new(1, 0);
      match self.fs_tree.find_by_ino(ino) {
        Some(vault_fs_tree::FsTree::Dir(dir)) => reply.attr(&ttl, &(self.dir_attr(dir.ino))),
        Some(vault_fs_tree::FsTree::Secrets(secrets)) => reply.attr(&ttl, &(self.dir_attr(secrets.ino))),
        Some(vault_fs_tree::FsTree::Secret(secret)) => reply.attr(&ttl, &(self.file_attr(secret.ino))),
        _ => reply.error(ENOENT),
      }
  }

  fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
      let mut entries = vec![
          (1, FileType::Directory, "."),
          (1, FileType::Directory, ".."),
      ];

      match self.fs_tree.find_by_ino(ino) {
        Some(entry) => match entry {
          vault_fs_tree::FsTree::Dir(dir) => {
            for dir_entry in &dir.data {
              match dir_entry {
                vault_fs_tree::FsTree::Dir(d) => entries.push((d.ino, FileType::Directory, &d.name.as_str()[0..d.name.len()-1])),
                vault_fs_tree::FsTree::Secrets(s) => entries.push((s.ino, FileType::Directory, s.name.as_str())),
                _ => panic!("shouldn't have secret in Dir: {:?}", dir),
              }
            }
          },
          vault_fs_tree::FsTree::Secrets(secrets) => {
            for secret in &secrets.data {
              match secret {
                vault_fs_tree::FsTree::Secret(s) => entries.push((s.ino, FileType::RegularFile, &s.name.as_str())),
                _ => panic!("shouldn't have non secret for secrets dir: {:?}", secrets),
              }
            }
          },
          vault_fs_tree::FsTree::Secret(secret) => panic!("shouldn't readdir for a secret! {:?}", secret),
        },
        None => println!("ino not found! {}\n{:?}", ino, self.fs_tree),
      }
      for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
          reply.add(entry.0, (i + 1) as i64, entry.1, entry.2);
      }
      reply.ok();
  }

  fn read(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, _size: u32, reply: ReplyData) {
    println!("read {}", ino);
    match self.fs_tree.find_by_ino(ino) {
      Some(vault_fs_tree::FsTree::Secret(secret)) => {
        reply.data(&secret.content.as_bytes()[offset as usize..]);
      },
      _ => reply.error(ENOENT),
    }
  }

  fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
    println!("lookup {}, {:?}", parent, name);
    let ttl = Timespec::new(1, 0);
    match self.fs_tree.find_by_ino_and_name(parent, name.to_str().unwrap().to_string()) {
      Some(vault_fs_tree::FsTree::Secret(secret)) => {
        reply.entry(&ttl, &self.file_attr(secret.ino), 0);
      },
      Some(vault_fs_tree::FsTree::Dir(dir)) => {
        reply.entry(&ttl, &self.dir_attr(dir.ino), 0);
      },
      Some(vault_fs_tree::FsTree::Secrets(secrets)) => {
        reply.entry(&ttl, &self.dir_attr(secrets.ino), 0);
      },
      _ => reply.error(ENOENT),
    }
  }
}
