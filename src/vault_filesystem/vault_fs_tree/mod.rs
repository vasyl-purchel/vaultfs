pub mod vault_api;

#[derive(Debug)]
pub struct FsDir {
  pub ino: u64,
  pub name: String,
  pub data: Vec<FsTree>,
}

// exactly same as a directory, but it doesn't use same API calls
#[derive(Debug)]
pub struct FsSecrets {
  pub ino: u64,
  pub name: String,
  pub data: Vec<FsTree>,
}

#[derive(Debug)]
pub struct FsSecret {
  pub ino: u64,
  pub name: String,
  pub content: String,
  pub created_time: String,
  pub updated_time: String,
}

#[derive(Debug)]
pub enum FsTree {
  Dir(FsDir),
  Secrets(FsSecrets),
  Secret(FsSecret),
}

fn build_tree(client: &vault_api::VaultClient, root_path: &str, tree: &mut FsTree, c: &mut u64) {
  match tree {
    FsTree::Dir(dir) => {
      match client.list_metadata(format!("{}{}", root_path, dir.name).as_str()) {
        Ok(metadata) => {
          for key in metadata.data.keys.iter() {
            *c = *c + 1;
            let name = key.as_str();
            if name.ends_with("/") {
              let mut entry = FsTree::Dir(FsDir {
                ino: *c,
                name: name[0..name.len()-1].to_string(),
                data: Vec::new(),
              });
              build_tree(client, format!("{}{}", root_path, dir.name).as_str(), &mut entry, c);
              dir.data.push(entry);
            } else {
              let mut entry = FsTree::Secrets(FsSecrets {
                ino: *c,
                name: name.to_string(),
                data: Vec::new(),
              });
              build_tree(client, format!("{}{}/", root_path, dir.name).as_str(), &mut entry, c);
              dir.data.push(entry);
            }
          }
        },
        err => println!("ups, metadata failed us! {:?}", err),
      }
    },
    FsTree::Secrets(secrets) => {
      match client.get_metadata(format!("{}{}", root_path, secrets.name).as_str()) {
        Ok(metadata) => {
          match client.get_data(format!("{}{}", root_path, secrets.name).as_str()) {
            Ok(data) => {
              for (secret_name, secret_value) in &data.data.data {
                *c = *c + 1;
                secrets.data.push(FsTree::Secret(FsSecret {
                  ino: *c,
                  name: secret_name.to_string(),
                  content: secret_value.to_string(),
                  created_time: metadata.data.created_time.to_string(),
                  updated_time: metadata.data.updated_time.to_string(),
                }))
              }
            },
            err => println!("ups, secrets data failed: {:?}", err),
          }
        },
        err => println!("ups, secrets metadata failed: {:?}", err),
      }
    },
    FsTree::Secret(secret) => println!("nothing to do to build a secret {:?}", secret),
  }
}

fn find_by_ino(tree: &FsTree, ino: u64) -> Option<&FsTree> {
  match tree {
    FsTree::Dir(dir) => {
      if dir.ino == ino {
        return Some(tree);
      }
      for entry in &dir.data {
        match find_by_ino(&entry, ino) {
          Some(node) => return Some(node),
          None => (),
        }
      }
      return None;
    },
    FsTree::Secrets(secrets) => {
      if secrets.ino == ino {
        return Some(tree);
      }
      for secret in &secrets.data {
        match find_by_ino(&secret, ino) {
          Some(node) => return Some(node),
          None => (),
        }
      }
      return None;
    },
    FsTree::Secret(secret) => {
      if secret.ino == ino {
        return Some(tree);
      } else {
        return None;
      }
    },
  }
}

fn find_by_ino_and_name(tree: &FsTree, ino: u64, sname: String) -> Option<&FsTree> {
  let name = &sname.as_str();
  match find_by_ino(tree, ino) {
    Some(FsTree::Dir(dir)) => {
      for entry in dir.data.iter() {
        match entry {
          FsTree::Dir(s) => {
            if s.name.as_str().contains(name) {
              return Some(entry);
            }
          },
          FsTree::Secrets(s) => {
            if s.name.as_str().contains(name) {
              return Some(entry);
            }
          },
          _ => println!("secret not in secrets?! {:?}", entry),
        }
      }
      return None;
    },
    Some(FsTree::Secrets(secrets)) => {
      for secret in secrets.data.iter() {
        match secret {
          FsTree::Secret(s) => {
            if s.name.as_str().contains(name) {
              return Some(secret);
            }
          },
          _ => println!("non secret in secrets?! {:?}", secret),
        }
      }
      return None;
    },
    Some(FsTree::Secret(secret)) => {
      println!("find_by_ino_and_name for secret {:?}", secret);
      return None;
    },
    None => return None,
  }
}

#[derive(Debug)]
pub struct VaultFsTree {
  root: FsTree,
}

impl VaultFsTree {
  pub fn new(client: &vault_api::VaultClient) -> VaultFsTree {
    let mut ino_counter = 1;
    let mut root = FsTree::Dir(FsDir {
      ino: ino_counter,
      name: "/".to_string(),
      data: Vec::new(),
    });
    build_tree(client, "", &mut root, &mut ino_counter);
    return VaultFsTree {
      root: root,
    };
  }

  pub fn find_by_ino(&self, ino: u64) -> Option<&FsTree> {
    return find_by_ino(&self.root, ino);
  }

  pub fn find_by_ino_and_name(&self, ino: u64, name: String) -> Option<&FsTree> {
    return find_by_ino_and_name(&self.root, ino, name);
  }
}
