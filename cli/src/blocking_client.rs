use std::sync::{Arc, Mutex};

use proto::jj_interface::{jujutsu_interface_client::JujutsuInterfaceClient, *};
use tokio::runtime::{Builder, Runtime};

type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;
type Result<T, E = StdError> = ::std::result::Result<T, E>;

// The order of the fields in this struct is important. They must be ordered
// such that when `BlockingJujutsuInterfaceClient` is dropped the client is dropped
// before the runtime. Not doing this will result in a deadlock when dropped.
// Rust drops struct fields in declaration order.
#[derive(Debug, Clone)]
pub struct BlockingJujutsuInterfaceClient {
    client: Arc<Mutex<JujutsuInterfaceClient<tonic::transport::Channel>>>,
    rt: Arc<Mutex<Runtime>>,
}

impl BlockingJujutsuInterfaceClient {
    pub fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
    where
        D: TryInto<tonic::transport::Endpoint>,
        D::Error: Into<StdError>,
    {
        let rt = Builder::new_multi_thread().enable_all().build().unwrap();
        let client = Arc::new(Mutex::new(
            rt.block_on(JujutsuInterfaceClient::connect(dst))?,
        ));
        let rt = Arc::new(Mutex::new(rt));

        Ok(Self { client, rt })
    }

    pub fn daemon_status(
        &self,
        request: impl tonic::IntoRequest<DaemonStatusReq>,
    ) -> Result<tonic::Response<DaemonStatusReply>, tonic::Status> {
        let mut client = self.client.lock().unwrap();
        let rt = self.rt.lock().unwrap();
        rt.block_on(client.daemon_status(request))
    }

    pub fn get_tree_state(
        &self,
        request: impl tonic::IntoRequest<GetTreeStateReq>,
    ) -> Result<tonic::Response<GetTreeStateReply>, tonic::Status> {
        let mut client = self.client.lock().unwrap();
        let rt = self.rt.lock().unwrap();
        rt.block_on(client.get_tree_state(request))
    }

    pub fn initialize(
        &self,
        request: impl tonic::IntoRequest<InitializeReq>,
    ) -> Result<tonic::Response<InitializeReply>, tonic::Status> {
        let mut client = self.client.lock().unwrap();
        let rt = self.rt.lock().unwrap();
        rt.block_on(client.initialize(request))
    }

    pub fn set_checkout_state(
        &self,
        request: impl tonic::IntoRequest<SetCheckoutStateReq>,
    ) -> Result<tonic::Response<SetCheckoutStateReply>, tonic::Status> {
        let mut client = self.client.lock().unwrap();
        let rt = self.rt.lock().unwrap();
        rt.block_on(client.set_checkout_state(request))
    }

    pub fn get_checkout_state(
        &self,
        request: impl tonic::IntoRequest<GetCheckoutStateReq>,
    ) -> Result<tonic::Response<CheckoutState>, tonic::Status> {
        let mut client = self.client.lock().unwrap();
        let rt = self.rt.lock().unwrap();
        rt.block_on(client.get_checkout_state(request))
    }

    pub fn snapshot(
        &self,
        request: impl tonic::IntoRequest<SnapshotReq>,
    ) -> Result<tonic::Response<SnapshotReply>, tonic::Status> {
        let mut client = self.client.lock().unwrap();
        let rt = self.rt.lock().unwrap();
        rt.block_on(client.snapshot(request))
    }

    pub fn write_commit(
        &self,
        request: impl tonic::IntoRequest<Commit>,
    ) -> Result<tonic::Response<CommitId>, tonic::Status> {
        let mut client = self.client.lock().unwrap();
        let rt = self.rt.lock().unwrap();
        rt.block_on(client.write_commit(request))
    }

    pub fn read_commit(
        &self,
        request: impl tonic::IntoRequest<CommitId>,
    ) -> Result<tonic::Response<Commit>, tonic::Status> {
        let mut client = self.client.lock().unwrap();
        let rt = self.rt.lock().unwrap();
        rt.block_on(client.read_commit(request))
    }

    pub fn write_file(
        &self,
        request: impl tonic::IntoRequest<File>,
    ) -> Result<tonic::Response<FileId>, tonic::Status> {
        let mut client = self.client.lock().unwrap();
        let rt = self.rt.lock().unwrap();
        rt.block_on(client.write_file(request))
    }

    pub fn read_file(
        &self,
        request: impl tonic::IntoRequest<FileId>,
    ) -> Result<tonic::Response<File>, tonic::Status> {
        let mut client = self.client.lock().unwrap();
        let rt = self.rt.lock().unwrap();
        rt.block_on(client.read_file(request))
    }

    pub fn write_tree(
        &self,
        request: impl tonic::IntoRequest<Tree>,
    ) -> Result<tonic::Response<TreeId>, tonic::Status> {
        let mut client = self.client.lock().unwrap();
        let rt = self.rt.lock().unwrap();
        rt.block_on(client.write_tree(request))
    }

    pub fn read_tree(
        &self,
        request: impl tonic::IntoRequest<TreeId>,
    ) -> Result<tonic::Response<Tree>, tonic::Status> {
        let mut client = self.client.lock().unwrap();
        let rt = self.rt.lock().unwrap();
        rt.block_on(client.read_tree(request))
    }

    pub fn write_symlink(
        &self,
        request: impl tonic::IntoRequest<Symlink>,
    ) -> Result<tonic::Response<SymlinkId>, tonic::Status> {
        let mut client = self.client.lock().unwrap();
        let rt = self.rt.lock().unwrap();
        rt.block_on(client.write_symlink(request))
    }

    pub fn read_symlink(
        &self,
        request: impl tonic::IntoRequest<SymlinkId>,
    ) -> Result<tonic::Response<Symlink>, tonic::Status> {
        let mut client = self.client.lock().unwrap();
        let rt = self.rt.lock().unwrap();
        rt.block_on(client.read_symlink(request))
    }

    pub fn get_empty_tree_id(&self) -> Result<tonic::Response<TreeId>, tonic::Status> {
        let rt = self.rt.lock().unwrap();
        let mut client = self.client.lock().unwrap();
        rt.block_on(client.get_empty_tree_id(GetEmptyTreeIdReq::default()))
    }
}
