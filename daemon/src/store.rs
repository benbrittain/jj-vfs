use jj_lib_proc_macros::ContentHash;

use crate::hash::blake3;

#[repr(transparent)]
#[derive(Clone, Debug, Default)]
pub struct Id([u8; 32]);

impl jj_lib::content_hash::ContentHash for Id {
    fn hash(&self, state: &mut impl digest::Update) {
        for x in self.0 {
            x.hash(state);
        }
    }
}

impl From<Vec<u8>> for Id {
    fn from(proto: Vec<u8>) -> Self {
        Id(proto.as_slice().try_into().expect("should fit in Id"))
    }
}

#[derive(Clone, Debug, Default)]
pub struct Symlink {
    // TODO maybe represent as PathBuf
    pub target: String,
}

#[derive(Clone, Debug, Default)]
pub struct File {
    pub content: Vec<u8>,
}

impl From<proto::jj_interface::Symlink> for Symlink {
    fn from(proto: proto::jj_interface::Symlink) -> Self {
        let mut symlink = Symlink::default();
        symlink.target = proto.target;
        symlink
    }
}

impl From<proto::jj_interface::File> for File {
    fn from(proto: proto::jj_interface::File) -> Self {
        let mut file = File::default();
        file.content = proto.data;
        file
    }
}

#[derive(Clone, Debug, Default, ContentHash)]
pub struct Tree {
    pub entries: Vec<TreeEntryMapping>,
}

#[derive(Clone, Debug, ContentHash)]
pub struct TreeEntryMapping {
    pub name: String,
    pub entry: TreeEntry,
}

impl Tree {
    pub fn get_hash(&self) -> Id {
        Id(*blake3(self).as_bytes())
    }

    pub fn as_proto(&self) -> proto::jj_interface::Tree {
        let mut proto = proto::jj_interface::Tree::default();
        for entry in &self.entries {
            let mut proto_entry = proto::jj_interface::tree::Entry::default();
            proto_entry.name = entry.name.clone();
            proto_entry.value = Some(entry.entry.as_proto());
            proto.entries.push(proto_entry);
        }
        proto
    }
}

impl From<proto::jj_interface::Tree> for Tree {
    fn from(proto: proto::jj_interface::Tree) -> Self {
        let mut tree = Tree::default();
        for proto_entry in proto.entries {
            let proto_val = proto_entry.value.unwrap();
            let entry = proto_val.into();
            tree.entries.push(TreeEntryMapping {
                name: proto_entry.name,
                entry,
            });
        }
        tree
    }
}

#[derive(Clone, Debug, ContentHash)]
pub enum TreeEntry {
    File { id: Id, executable: bool },
    TreeId(Id),
    SymlinkId(Id),
    ConflictId(Id),
}

impl From<proto::jj_interface::TreeValue> for TreeEntry {
    fn from(proto: proto::jj_interface::TreeValue) -> Self {
        let value: proto::jj_interface::tree_value::Value = proto.value.unwrap();
        use proto::jj_interface::tree_value::Value::*;
        match value {
            TreeId(id) => TreeEntry::TreeId(id.into()),
            SymlinkId(id) => TreeEntry::SymlinkId(id.into()),
            ConflictId(id) => TreeEntry::ConflictId(id.into()),
            File(file) => TreeEntry::File {
                id: file.id.try_into().unwrap(),
                executable: file.executable,
            },
        }
    }
}

impl TreeEntry {
    pub fn as_proto(&self) -> proto::jj_interface::TreeValue {
        let mut proto = proto::jj_interface::TreeValue::default();
        proto.value = Some(match self {
            TreeEntry::File { id, executable } => {
                let mut proto_entry = proto::jj_interface::tree_value::File::default();
                proto_entry.id = id.0.to_vec();
                proto_entry.executable = *executable;
                proto::jj_interface::tree_value::Value::File(proto_entry)
            }
            _ => todo!(),
        });
        proto
    }
}
/// Stores mount-agnostic information like Trees or Commits. Unaware of filesystem information.
#[derive(Clone, Debug)]
pub struct Store {}

impl Store {
    pub fn new() -> Self {
        Store {}
    }

    pub async fn get_tree(&self, _id: Id) -> Option<Tree> {
        todo!()
    }

    #[tracing::instrument]
    pub async fn write_tree(&self, _tree: Tree) -> Id {
        todo!()
    }

    pub async fn get_file(&self, _id: Id) -> Option<File> {
        todo!()
    }

    #[tracing::instrument]
    pub async fn write_file(&self, _file: File) -> Id {
        todo!()
    }

    pub async fn get_symlink(&self, _id: Id) -> Option<Symlink> {
        todo!()
    }

    #[tracing::instrument]
    pub async fn write_symlink(&self, _symlink: Symlink) -> Id {
        todo!()
    }
}
