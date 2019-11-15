extern crate fuse;
extern crate libc;
extern crate env_logger;
extern crate time;
extern crate clap;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

#[macro_use] extern crate serde_derive;

use std::ffi::OsStr;
use clap::{Arg, App};

mod vault_filesystem;
mod settings;

use vault_filesystem::VaultFilesystem;
use vault_filesystem::vault_fs_tree::VaultFsTree;
use vault_filesystem::vault_fs_tree::vault_api::VaultClient;
use settings::Settings;

fn main() {
    let matches = App::new("vaultfs")
        .version("1.0")
        .about("Mounts vault secrets as a folder")
        .author("Vasyl Purchel<vasyl.purchel@gmail.com>")
        .arg(Arg::with_name("mount_path")
                .required(true)
                .takes_value(true)
                .index(1)
                .help("path to mount vault fuse file system at"))
        .arg(Arg::with_name("config")
                .required(true)
                .takes_value(true)
                .index(2)
                .help("path to a config file"))
        .get_matches();
    let mount_path = matches.value_of("mount_path").unwrap();
    let config_path = matches.value_of("config").unwrap();
    let settings = Settings::new(config_path).unwrap();
    let client = VaultClient::new(settings.vault.address, settings.vault.token);

    let options = ["-o", "ro", "-o", "fsname=hello"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    let tree = VaultFsTree::new(&client);
    fuse::mount(VaultFilesystem::new(tree), &mount_path, &options).unwrap();
}
