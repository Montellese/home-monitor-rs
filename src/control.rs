use std::path::Path;
use std::sync::Arc;

use crate::dom::Server;
use crate::networking::{
    FastPinger, Pinger, ShutdownServer, Ssh2ShutdownServer, WakeOnLanServer, WakeupServer,
};
use crate::utils::{AlwaysOff, AlwaysOffFile, AlwaysOn, AlwaysOnFile};

#[derive(Clone)]
pub struct ServerControl {
    pub server: Server,
    pub wakeup: Arc<dyn WakeupServer>,
    pub shutdown: Arc<dyn ShutdownServer>,

    pub always_off: Arc<dyn AlwaysOff>,
    pub always_on: Arc<dyn AlwaysOn>,
}

pub struct Factory {}

impl Factory {
    pub fn create_pinger(max_rtt: Option<u64>) -> Box<dyn Pinger> {
        Box::new(FastPinger::new(max_rtt))
    }

    pub fn create_shutdown_server(server: &Server) -> Arc<dyn ShutdownServer> {
        Arc::new(Ssh2ShutdownServer::new(server))
    }

    pub fn create_wakeup_server(server: &Server) -> Arc<dyn WakeupServer> {
        Arc::new(WakeOnLanServer::new(server))
    }

    pub fn create_always_off(root_path: &Path, server: &Server) -> Arc<dyn AlwaysOff> {
        let mut path = root_path.to_path_buf();
        path.push(server.machine.id.to_string());
        Arc::new(AlwaysOffFile::new(&path))
    }

    pub fn create_always_on(root_path: &Path, server: &Server) -> Arc<dyn AlwaysOn> {
        let mut path = root_path.to_path_buf();
        path.push(server.machine.id.to_string());
        Arc::new(AlwaysOnFile::new(&path))
    }

    pub fn create_control(server: &Server, files_api_root_path: &Path) -> ServerControl {
        ServerControl {
            server: server.clone(),
            wakeup: Self::create_wakeup_server(server),
            shutdown: Self::create_shutdown_server(server),
            always_off: Self::create_always_off(files_api_root_path, server),
            always_on: Self::create_always_on(files_api_root_path, server),
        }
    }
}

#[cfg(test)]
pub mod test {
    use rstest::*;

    use super::*;
    use crate::dom::device::test::*;

    pub struct MockServerControl {
        pub server: Server,
        pub wakeup: crate::networking::MockWakeupServer,
        pub shutdown: crate::networking::MockShutdownServer,

        pub always_off: crate::utils::MockAlwaysOff,
        pub always_on: crate::utils::MockAlwaysOn,
    }

    impl From<MockServerControl> for ServerControl {
        fn from(mock_server_control: MockServerControl) -> Self {
            Self {
                server: mock_server_control.server,
                wakeup: Arc::new(mock_server_control.wakeup),
                shutdown: Arc::new(mock_server_control.shutdown),
                always_off: Arc::new(mock_server_control.always_off),
                always_on: Arc::new(mock_server_control.always_on),
            }
        }
    }

    #[fixture]
    pub fn mocked_server_control(server: Server) -> MockServerControl {
        MockServerControl {
            server: server,
            wakeup: crate::networking::MockWakeupServer::new(),
            shutdown: crate::networking::MockShutdownServer::new(),
            always_off: crate::utils::MockAlwaysOff::new(),
            always_on: crate::utils::MockAlwaysOn::new(),
        }
    }
}
