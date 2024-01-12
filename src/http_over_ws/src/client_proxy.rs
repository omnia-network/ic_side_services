use std::collections::BTreeMap;
use crate::http_connection::{HttpConnectionId, HttpConnection, HttpFailureReason};

pub(crate) struct ClientProxy {
    connections: BTreeMap<HttpConnectionId, HttpConnection>,
}

impl ClientProxy {
    pub fn new() -> Self {
        ClientProxy {
            connections: BTreeMap::new(),
        }
    }

    pub fn assign_connection(
        &mut self,
        connection_id: HttpConnectionId,
        connection: HttpConnection,
    ) {
        self.connections.insert(connection_id, connection);
    }

    pub fn report_connection_failure(&mut self, connection_id: HttpConnectionId, reason: HttpFailureReason) {
        self.connections.get_mut(&connection_id)
            .and_then(|r| {
                r.report_failure(reason);
                Some(r.clone())
            });
    }

    pub fn get_request_mut(&mut self, connection_id: HttpConnectionId) -> Result<&mut HttpConnection, HttpFailureReason> {
        self.connections.get_mut(&connection_id).ok_or(
            HttpFailureReason::RequestIdNotFound,
            )
    }

    pub fn get_connections(&self) -> &BTreeMap<HttpConnectionId, HttpConnection> {
        &self.connections
    }

    pub fn remove_connection(&mut self, connection_id: HttpConnectionId) -> Result<HttpConnection, HttpFailureReason> {
        self.connections.remove(&connection_id).ok_or(HttpFailureReason::ProxyError(String::from(
            "client has not been assigned the request",
        )))
    }
}