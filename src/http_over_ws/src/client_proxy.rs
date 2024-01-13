use std::collections::BTreeMap;
use crate::http_connection::{HttpRequestId, HttpConnection, HttpFailureReason};

pub(crate) struct ClientProxy {
    connections: BTreeMap<HttpRequestId, HttpConnection>,
}

impl ClientProxy {
    pub(crate) fn new() -> Self {
        ClientProxy {
            connections: BTreeMap::new(),
        }
    }

    pub(crate) fn assign_connection(
        &mut self,
        request_id: HttpRequestId,
        connection: HttpConnection,
    ) {
        self.connections.insert(request_id, connection);
    }

    pub(crate) fn report_connection_failure(&mut self, request_id: HttpRequestId, reason: HttpFailureReason) {
        self.connections.get_mut(&request_id)
            .and_then(|connection| {
                connection.report_failure(reason);
                Some(connection)
            });
    }

    pub(crate) fn get_connection_mut(&mut self, request_id: HttpRequestId) -> Result<&mut HttpConnection, HttpFailureReason> {
        self.connections.get_mut(&request_id).ok_or(
            HttpFailureReason::RequestIdNotFound,
            )
    }

    pub(crate) fn get_connections(&self) -> &BTreeMap<HttpRequestId, HttpConnection> {
        &self.connections
    }

    pub(crate) fn remove_connection(&mut self, request_id: HttpRequestId) -> Result<HttpConnection, HttpFailureReason> {
        self.connections.remove(&request_id).ok_or(HttpFailureReason::ProxyError(String::from(
            "client has not been assigned the connection",
        )))
    }
}