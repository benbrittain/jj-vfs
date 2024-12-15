use prost::Message;
use proto::jj_interface::*;
use tonic::{Request, Response, Status};
use tracing::info;

use crate::{store::Store, vfs_mgr::VfsManagerHandle};

pub struct JujutsuService {
    vfs_handle: VfsManagerHandle,
    store: Store,
}

impl JujutsuService {
    pub fn new(
        vfs_handle: VfsManagerHandle,
    ) -> jujutsu_interface_server::JujutsuInterfaceServer<Self> {
        jujutsu_interface_server::JujutsuInterfaceServer::new(JujutsuService {
            vfs_handle,
            store: Store::new(),
        })
    }
}

#[tonic::async_trait]
impl jujutsu_interface_server::JujutsuInterface for JujutsuService {
    #[tracing::instrument(skip(self))]
    async fn initialize(
        &self,
        request: Request<InitializeReq>,
    ) -> Result<Response<InitializeReply>, Status> {
        let req = request.into_inner();
        info!("Initializing a new repo at {}", req.path);
        // TODO: This needs to handle the error
        let _resp = self.vfs_handle.bind();
        Ok(Response::new(InitializeReply {}))
    }

    #[tracing::instrument(skip(self))]
    async fn get_tree_state(
        &self,
        request: Request<GetTreeStateReq>,
    ) -> Result<Response<GetTreeStateReply>, Status> {
        info!("Getting tree state");
        let _req = request.into_inner();
        todo!()
    }

    #[tracing::instrument(skip(self))]
    async fn get_checkout_state(
        &self,
        request: Request<GetCheckoutStateReq>,
    ) -> Result<Response<CheckoutState>, Status> {
        info!("Getting checkout state");
        let _req = request.into_inner();
        todo!()
    }

    #[tracing::instrument(skip(self))]
    async fn set_checkout_state(
        &self,
        request: Request<SetCheckoutStateReq>,
    ) -> Result<Response<SetCheckoutStateReply>, Status> {
        let _req = request.into_inner();
        todo!()
    }

    #[tracing::instrument(skip(self))]
    async fn snapshot(
        &self,
        request: Request<SnapshotReq>,
    ) -> Result<Response<SnapshotReply>, Status> {
        let _req = request.into_inner();
        todo!()
    }

    #[tracing::instrument(skip(self))]
    async fn get_empty_tree_id(
        &self,
        _request: Request<GetEmptyTreeIdReq>,
    ) -> Result<Response<TreeId>, Status> {
        Ok(Response::new(TreeId {
            tree_id: b"00000000".to_vec(),
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn concurrency(
        &self,
        _request: Request<ConcurrencyRequest>,
    ) -> Result<Response<ConcurrencyReply>, Status> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    async fn write_file(&self, _request: Request<File>) -> Result<Response<FileId>, Status> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    async fn read_file(&self, _request: Request<FileId>) -> Result<Response<File>, Status> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    async fn write_symlink(
        &self,
        request: Request<Symlink>,
    ) -> Result<Response<SymlinkId>, Status> {
        let symlink = request.into_inner();
        let _symlink_id = *blake3::hash(&symlink.encode_to_vec()).as_bytes();
        todo!()
    }

    #[tracing::instrument(skip(self))]
    async fn read_symlink(&self, request: Request<SymlinkId>) -> Result<Response<Symlink>, Status> {
        let _symlink_id = request.into_inner();
        todo!()
    }

    #[tracing::instrument(skip(self))]
    async fn write_tree(&self, request: Request<Tree>) -> Result<Response<TreeId>, Status> {
        let tree = request.into_inner();
        let tree_id = blake3::hash(&tree.encode_to_vec());
        //self.trees.insert(
        //    TreeId {
        //        tree_id: tree_id.to_hex().as_bytes().to_vec(),
        //    },
        //    tree,
        //);
        todo!()
    }

    #[tracing::instrument(skip(self))]
    async fn read_tree(&self, _request: Request<TreeId>) -> Result<Response<Tree>, Status> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    async fn write_commit(&self, request: Request<Commit>) -> Result<Response<CommitId>, Status> {
        let _commit = request.into_inner();
        todo!()
    }

    #[tracing::instrument(skip(self))]
    async fn read_commit(&self, _request: Request<CommitId>) -> Result<Response<Commit>, Status> {
        todo!()
    }
}
