use std::{collections::HashMap, time::Duration, cell::RefCell};
use candid::Principal;

use crate::{http_connection::{HttpFailureReason, HttpConnectionId, HttpCallback, HttpRequest, HttpRequestTimeoutMs, HttpResponse, GetHttpResponseResult, HttpConnection}, client_proxy::ClientProxy};


// local state
thread_local! {
    /* flexible */ pub static STATE: RefCell<State> = RefCell::new(State::new());
}

pub struct State {
    connected_clients: ConnectedClients,
    next_connection_id: HttpConnectionId,
}

impl State {
    pub fn new() -> Self {
        State {
            connected_clients: ConnectedClients::new(),
            next_connection_id: 0,
        }
    }

    pub fn add_client(&mut self, client_principal: Principal) {
        self.connected_clients.add_client(client_principal);
    }


    pub fn remove_client(&mut self, client_principal: &Principal) -> Result<(), HttpFailureReason> {
        self.connected_clients.remove_client(client_principal)
    }

    pub fn assign_connection(
        &mut self,
        connection: HttpRequest,
        callback: Option<HttpCallback>,
        timeout_ms: Option<HttpRequestTimeoutMs>,
    ) -> Result<(Principal, HttpConnectionId), HttpFailureReason> {
        let connection_id = self.next_connection_id();

        let client_principal = self
            .get_client_for_connection(connection_id)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "no clients connected",
            )))?
            .clone();

        let timer_id = timeout_ms.and_then(|millis| {
            Some(ic_cdk_timers::set_timer(
                Duration::from_millis(millis),
                move || {
                    http_connection_timeout(client_principal, connection_id);
                },
            ))
        });

        let connection = HttpConnection::new(
            connection_id,
            connection,
            callback,
            timer_id,
        );

        self.connected_clients.assign_connection_to_client(&client_principal, connection_id, connection)?;
        Ok((client_principal, connection_id))
    }

    fn next_connection_id(&mut self) -> HttpConnectionId {
        self.next_connection_id += 1;
        self.next_connection_id
    } 

    fn get_client_for_connection(&self, connection_id: HttpConnectionId) -> Option<Principal> {
        let connected_clients_count = self.connected_clients.0.len();
        if connected_clients_count == 0 {
            return None;
        }
        let chosen_client_index = connection_id as usize % connected_clients_count;
        // chosen_client_index is in [0, connected_clients_count)
        // where connected_clients_count is the number of clients currently connected.
        // as no client is removed while executing this method,
        // the entry at 'chosen_client_index' is guaranteed to exist
        Some(
            self.connected_clients
                .0
                .iter()
                .nth(chosen_client_index)
                .expect("client is not connected")
                .0
                .clone(),
        )
    }

    pub fn report_connection_failure(&mut self, client_principal: Principal, connection_id: HttpConnectionId, reason: HttpFailureReason) {
        self.connected_clients
            .0
            .get_mut(&client_principal)
            .and_then(|client| {
                client.report_connection_failure(connection_id, reason);
                Some(client)
            });
    }

    pub fn handle_http_response(&mut self, client_principal: Principal, connection_id: HttpConnectionId, response: HttpResponse) -> Result<(), HttpFailureReason> {
        let client = self.connected_clients
            .0
            .get_mut(&client_principal)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "proxy not connected",
            )))?;
        let connection = client.get_connection_mut(connection_id)?;
        
        connection.update_state(response)
    }

    pub fn get_http_connection(&self, connection_id: HttpConnectionId) -> Option<HttpRequest> {
        for (_, proxy) in
            self
                .connected_clients
                .0
                .iter() 
        {
            for (id, connection) in proxy.get_connections() {
                if id.to_owned() == connection_id {
                    return Some(connection.get_request());
                }
            }
        }
        None
    }

    pub fn get_http_response(&self, connection_id: HttpConnectionId) -> GetHttpResponseResult {
        for (_, proxy) in
            self
                .connected_clients
                .0
                .iter() 
        {
            for (id, connection) in proxy.get_connections() {
                if id.to_owned() == connection_id {
                    return connection.get_response();
                }
            }
        }
        Err(HttpFailureReason::RequestIdNotFound)
    }
}


fn http_connection_timeout(client_principal: Principal, connection_id: HttpConnectionId) {
    STATE.with(|state| {
        state.borrow_mut()
            .connected_clients
            .0
            .get_mut(&client_principal)
            .and_then(|client| {
                let r = client.get_connection_mut(connection_id).and_then(|connection| {
                    connection.set_timeout();
                    Ok(connection)
                });
                Some(r)
            });
        // if let Err(_) = state
        //     .borrow_mut()
        //     .connected_clients
        //     .complete_connection_for_client(client_principal, connection_id)
        // {
        //     log("cannot complete connection");
        // }
    });
}

pub(crate) struct ConnectedClients(HashMap<Principal, ClientProxy>);

impl ConnectedClients {
    fn new() -> Self {
        ConnectedClients(HashMap::new())
    }

    fn add_client(&mut self, client_principal: Principal) {
        self.0.insert(client_principal, ClientProxy::new());
    }

    fn assign_connection_to_client(
        &mut self,
        client_principal: &Principal,
        connection_id: HttpConnectionId,
        connection: HttpConnection,
    ) -> Result<(), HttpFailureReason> {
        let proxy = self.0
            .get_mut(&client_principal)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "proxy not connected",
            )))?;
        proxy.assign_connection(connection_id, connection);
        Ok(())
    }

    fn complete_connection_for_client(
        &mut self,
        client_principal: Principal,
        connection_id: HttpConnectionId,
    ) -> Result<(), HttpFailureReason> {
        let client = self
            .0
            .get_mut(&client_principal)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "proxy not connected",
            )))?;
        client.remove_connection(connection_id)?;
        Ok(())
    }

    fn remove_client(&mut self, client_principal: &Principal) -> Result<(), HttpFailureReason> {
        self.0
            .remove(client_principal)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "client not connected",
            )))?;
        Ok(())
    }
}



// #[cfg(test)]
// mod tests {
//     use candid::Principal;

//     use super::*;

//     #[test]
//     fn should_add_client_and_assign_connection() {
//         let mut clients = ConnectedClients::new();
//         let client_principal = Principal::anonymous();
//         clients.add_client(client_principal);
//         assert_eq!(clients.0.len(), 1);

//         let connection_id = 1;
//         assert!(clients.assign_connection(connection_id).is_ok());
//         assert!(clients
//             .0
//             .get(&client_principal)
//             .expect("client is not connected")
//             .contains(&connection_id));
//     }

//     #[test]
//     fn should_not_assign_connection() {
//         let mut clients = ConnectedClients::new();
//         assert!(clients.assign_connection(1).is_err());
//     }

//     #[test]
//     fn should_complete_connection() {
//         let mut clients = ConnectedClients::new();

//         let client_principal = Principal::anonymous();
//         clients.add_client(client_principal);
//         let connection_id = 1;
//         assert!(clients.assign_connection(connection_id).is_ok());
//         assert!(clients
//             .complete_connection_for_client(client_principal, connection_id)
//             .is_ok());
//     }

//     #[test]
//     fn should_distribute_connections_among_clients() {
//         let mut clients = ConnectedClients::new();

//         let client_principal = Principal::from_text("aaaaa-aa").unwrap();
//         let another_client_principal = Principal::from_text("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();

//         clients.add_client(client_principal);
//         clients.add_client(another_client_principal);

//         let connection_id = 1;
//         assert!(clients.assign_connection(connection_id).is_ok());

//         let connection_id = 2;
//         assert!(clients.assign_connection(connection_id).is_ok());

//         let connection_id = 3;
//         assert!(clients.assign_connection(connection_id).is_ok());

//         let connection_id = 4;
//         assert!(clients.assign_connection(connection_id).is_ok());

//         assert!(
//             clients
//                 .0
//                 .get(&client_principal)
//                 .expect("client is not connected")
//                 .len()
//                 == 2
//         );
//         assert!(
//             clients
//                 .0
//                 .get(&another_client_principal)
//                 .expect("client is not connected")
//                 .len()
//                 == 2
//         );
//     }
// }
