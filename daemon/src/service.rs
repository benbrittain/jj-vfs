use proto::jj_interface::*;
use tonic::{Request, Response, Status};
use tracing::info;

use crate::{store::Store, ty::Id};

pub struct JujutsuService {
    store: Store,
}

impl JujutsuService {
    pub fn new() -> jujutsu_interface_server::JujutsuInterfaceServer<Self> {
        jujutsu_interface_server::JujutsuInterfaceServer::new(JujutsuService {
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
        Ok(Response::new(InitializeReply {}))
    }

    #[tracing::instrument(skip(self))]
    async fn get_empty_tree_id(
        &self,
        _request: Request<GetEmptyTreeIdReq>,
    ) -> Result<Response<TreeId>, Status> {
        Ok(Response::new(TreeId {
            tree_id: self.store.get_empty_tree_id().into(),
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn write_file(&self, request: Request<File>) -> Result<Response<FileId>, Status> {
        let file = request.into_inner();
        let file_id = self.store.write_file(file.into()).await.into();
        Ok(Response::new(FileId { file_id }))
    }

    #[tracing::instrument(skip(self))]
    async fn read_file(&self, request: Request<FileId>) -> Result<Response<File>, Status> {
        let file_id: Id = request.into_inner().into();
        let file = self.store.get_file(file_id).unwrap();
        Ok(Response::new(file.as_proto()))
    }

    #[tracing::instrument(skip(self))]
    async fn write_symlink(
        &self,
        request: Request<Symlink>,
    ) -> Result<Response<SymlinkId>, Status> {
        let symlink = request.into_inner();
        let symlink_id = self.store.write_symlink(symlink.into()).await.into();
        Ok(Response::new(SymlinkId { symlink_id }))
    }

    #[tracing::instrument(skip(self))]
    async fn read_symlink(&self, request: Request<SymlinkId>) -> Result<Response<Symlink>, Status> {
        let symlink_id: Id = request.into_inner().into();
        let symlink = self.store.get_symlink(symlink_id).unwrap();
        Ok(Response::new(symlink.as_proto()))
    }

    #[tracing::instrument(skip(self))]
    async fn write_tree(&self, request: Request<Tree>) -> Result<Response<TreeId>, Status> {
        let tree = request.into_inner();
        let tree_id = self.store.write_tree(tree.into()).await.into();
        Ok(Response::new(TreeId { tree_id }))
    }

    #[tracing::instrument(skip(self))]
    async fn read_tree(&self, request: Request<TreeId>) -> Result<Response<Tree>, Status> {
        let tree_id: Id = request.into_inner().into();
        let tree = self.store.get_tree(tree_id).unwrap();
        Ok(Response::new(tree.as_proto()))
    }

    #[tracing::instrument(skip(self))]
    async fn write_commit(&self, request: Request<Commit>) -> Result<Response<CommitId>, Status> {
        let commit = request.into_inner();
        if commit.parents.is_empty() {
            return Err(Status::internal("Cannot write a commit with no parents"));
        }
        let commit_id = self.store.write_commit(commit.into()).await.into();
        Ok(Response::new(CommitId { commit_id }))
    }

    #[tracing::instrument(skip(self))]
    async fn read_commit(&self, request: Request<CommitId>) -> Result<Response<Commit>, Status> {
        let commit_id: Id = request.into_inner().into();
        let commits = self.store.commits.lock();
        let commit = commits.get(&commit_id).unwrap();
        Ok(Response::new((*commit).as_proto()))
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
}

#[cfg(test)]
mod tests {
    const COMMIT_ID_LENGTH: usize = 32;
    const CHANGE_ID_LENGTH: usize = 16;

    use assert_matches::assert_matches;
    use proto::jj_interface::jujutsu_interface_server::JujutsuInterface;

    use super::*;

    #[tokio::test]
    async fn write_commit_parents() {
        let svc = JujutsuService {
            store: Store::new(),
        };
        let mut commit = Commit::default();

        // No parents
        commit.parents = vec![];

        assert_matches!(
            svc.write_commit(Request::new(commit.clone())).await,
            Err(status) if status.message().contains("no parents")
        );

        // Only root commit as parent
        commit.parents = vec![vec![0; CHANGE_ID_LENGTH]];
        let first_id = svc
            .write_commit(Request::new(commit.clone()))
            .await
            .unwrap()
            .into_inner();
        let first_commit = svc
            .read_commit(Request::new(first_id.clone()))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(first_commit, commit);

        // Only non-root commit as parent
        commit.parents = vec![first_id.clone().commit_id];
        let second_id = svc
            .write_commit(Request::new(commit.clone()))
            .await
            .unwrap()
            .into_inner();
        let second_commit = svc
            .read_commit(Request::new(second_id.clone()))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(second_commit, commit);

        // Merge commit
        commit.parents = vec![first_id.clone().commit_id, second_id.commit_id];
        let merge_id = svc
            .write_commit(Request::new(commit.clone()))
            .await
            .unwrap()
            .into_inner();
        let merge_commit = svc
            .read_commit(Request::new(merge_id.clone()))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(merge_commit, commit);

        commit.parents = vec![first_id.commit_id, vec![0; COMMIT_ID_LENGTH]];
        let root_merge_id = svc
            .write_commit(Request::new(commit.clone()))
            .await
            .unwrap()
            .into_inner();
        let root_merge_commit = svc
            .read_commit(Request::new(root_merge_id.clone()))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(root_merge_commit, commit);
    }
}
