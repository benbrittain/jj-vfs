use jj_lib_proc_macros::ContentHash;

use crate::hash::blake3;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Id(pub [u8; 32]);

impl jj_lib::content_hash::ContentHash for Id {
    fn hash(&self, state: &mut impl digest::Update) {
        for x in self.0 {
            x.hash(state);
        }
    }
}

impl Into<Vec<u8>> for Id {
    fn into(self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl From<proto::jj_interface::FileId> for Id {
    fn from(proto: proto::jj_interface::FileId) -> Self {
        proto.file_id.into()
    }
}

impl From<proto::jj_interface::CommitId> for Id {
    fn from(proto: proto::jj_interface::CommitId) -> Self {
        proto.commit_id.into()
    }
}

impl From<proto::jj_interface::SymlinkId> for Id {
    fn from(proto: proto::jj_interface::SymlinkId) -> Self {
        proto.symlink_id.into()
    }
}

impl From<proto::jj_interface::TreeId> for Id {
    fn from(proto: proto::jj_interface::TreeId) -> Self {
        proto.tree_id.into()
    }
}

impl From<Vec<u8>> for Id {
    fn from(proto: Vec<u8>) -> Self {
        Id(proto.as_slice().try_into().expect("should fit in Id"))
    }
}

#[derive(Clone, Debug, Default, ContentHash)]
pub struct Symlink {
    // TODO: maybe represent as PathBuf
    pub target: String,
}

impl Symlink {
    pub fn get_hash(&self) -> Id {
        Id(*blake3(self).as_bytes())
    }

    pub fn as_proto(&self) -> proto::jj_interface::Symlink {
        let mut proto = proto::jj_interface::Symlink::default();
        proto.target = self.target.clone();
        proto
    }
}

impl From<proto::jj_interface::Symlink> for Symlink {
    fn from(proto: proto::jj_interface::Symlink) -> Self {
        let mut symlink = Symlink::default();
        symlink.target = proto.target;
        symlink
    }
}

#[derive(Clone, Debug, Default, ContentHash)]
pub struct CommitTimestamp {
    millis_since_epoch: i64,
    tz_offset: i32,
}
impl CommitTimestamp {
    pub fn as_proto(&self) -> proto::jj_interface::commit::Timestamp {
        let mut proto = proto::jj_interface::commit::Timestamp::default();
        proto.millis_since_epoch = self.millis_since_epoch;
        proto.tz_offset = self.tz_offset;
        proto
    }
}

impl CommitSignature {
    pub fn as_proto(&self) -> proto::jj_interface::commit::Signature {
        let mut proto = proto::jj_interface::commit::Signature::default();
        proto.name = self.name.clone();
        proto.email = self.email.clone();
        proto.timestamp = Some(self.timestamp.as_proto());
        proto
    }
}

impl From<proto::jj_interface::commit::Timestamp> for CommitTimestamp {
    fn from(proto: proto::jj_interface::commit::Timestamp) -> Self {
        let mut ts = CommitTimestamp::default();
        ts.millis_since_epoch = proto.millis_since_epoch;
        ts.tz_offset = proto.tz_offset;
        ts
    }
}

impl From<proto::jj_interface::commit::Signature> for CommitSignature {
    fn from(proto: proto::jj_interface::commit::Signature) -> Self {
        let mut sig = CommitSignature::default();
        sig.name = proto.name;
        sig.email = proto.email;
        sig.timestamp = proto.timestamp.unwrap().into();
        sig
    }
}

#[derive(Clone, Debug, Default, ContentHash)]
pub struct CommitSignature {
    name: String,
    email: String,
    timestamp: CommitTimestamp,
}

#[derive(Clone, Debug, Default, ContentHash)]
pub struct Commit {
    pub parents: Vec<Vec<u8>>,
    pub predecessors: Vec<Vec<u8>>,
    pub root_tree: Vec<Vec<u8>>,
    pub uses_tree_conflict_format: bool,
    pub change_id: Vec<u8>,
    pub description: String,
    pub author: Option<CommitSignature>,
    pub committer: Option<CommitSignature>,
}

impl Commit {
    pub fn get_hash(&self) -> Id {
        Id(*blake3(self).as_bytes())
    }

    pub fn as_proto(&self) -> proto::jj_interface::Commit {
        let mut proto = proto::jj_interface::Commit::default();
        proto.parents = self.parents.clone();
        proto.predecessors = self.predecessors.clone();
        proto.root_tree = self.root_tree.clone();
        proto.uses_tree_conflict_format = self.uses_tree_conflict_format.clone();
        proto.change_id = self.change_id.clone();
        proto.description = self.description.clone();
        proto.author = self.author.clone().map(|a| a.as_proto());
        proto.committer = self.committer.clone().map(|a| a.as_proto());
        proto
    }
}

impl From<proto::jj_interface::Commit> for Commit {
    fn from(proto: proto::jj_interface::Commit) -> Self {
        let mut commit = Commit::default();
        commit.parents = proto.parents;
        commit.predecessors = proto.predecessors;
        commit.root_tree = proto.root_tree.clone();
        commit.uses_tree_conflict_format = proto.uses_tree_conflict_format.clone();
        commit.change_id = proto.change_id.clone();
        commit.description = proto.description.clone();
        commit.author = proto.author.map(Into::into);
        commit.committer = proto.committer.map(Into::into);
        commit
    }
}

#[derive(Clone, Debug, Default, ContentHash)]
pub struct File {
    pub content: Vec<u8>,
}

impl File {
    pub fn get_hash(&self) -> Id {
        Id(*blake3(self).as_bytes())
    }

    pub fn as_proto(&self) -> proto::jj_interface::File {
        let mut proto = proto::jj_interface::File::default();
        proto.data = self.content.clone();
        proto
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
