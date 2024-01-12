use std::{collections::{HashMap, BTreeMap}, time::Duration};

use candid::Principal;
use logger::log;

use crate::{types::{HttpFailureReason, HttpRequestId}, HttpRequestState, HttpCallback, HttpRequest, HttpRequestTimeoutMs, STATE, HttpResponse, GetHttpResponseResult};

#[derive(Clone)]
pub(crate) struct ConnectedClients(pub HashMap<Principal, BTreeMap<HttpRequestId, HttpRequestState>>);

impl ConnectedClients {
    fn new() -> Self {
        ConnectedClients(HashMap::new())
    }

    fn add_client(&mut self, client_principal: Principal) {
        self.0.insert(client_principal, BTreeMap::new());
    }

    fn assign_request_to_client(
        &mut self,
        client_principal: &Principal,
        request_id: HttpRequestId,
        request: HttpRequestState,
    ) -> Result<(), HttpFailureReason> {
        self.0
            .get_mut(&client_principal)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "proxy not connected",
            )))?
            .insert(request_id, request);
        Ok(())
    }

    fn complete_request_for_client(
        &mut self,
        client_principal: Principal,
        request_id: HttpRequestId,
    ) -> Result<(), HttpFailureReason> {
        let client = self
            .0
            .get_mut(&client_principal)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "proxy not connected",
            )))?;
        client
            .remove(&request_id)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "client has not been assigned the request",
            )))?;
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

pub struct State {
    connected_clients: ConnectedClients,
    next_request_id: HttpRequestId,
}

impl State {
    pub fn new() -> Self {
        State {
            connected_clients: ConnectedClients::new(),
            next_request_id: 0,
        }
    }

    pub fn add_client(&mut self, client_principal: Principal) {
        self.connected_clients.add_client(client_principal);
    }


    pub fn remove_client(&mut self, client_principal: &Principal) -> Result<(), HttpFailureReason> {
        self.connected_clients.remove_client(client_principal)
    }

    pub fn assign_request(
        &mut self,
        request: HttpRequest,
        callback: Option<HttpCallback>,
        timeout_ms: Option<HttpRequestTimeoutMs>,
    ) -> Result<(Principal, HttpRequestId), HttpFailureReason> {
        let request_id = self.next_request_id();

        let client_principal = self
            .get_client_for_request(request_id)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "no clients connected",
            )))?
            .clone();

        let timer_id = timeout_ms.and_then(|millis| {
            Some(ic_cdk_timers::set_timer(
                Duration::from_millis(millis),
                move || {
                    http_request_timeout(client_principal, request_id);
                },
            ))
        });

        let request = HttpRequestState::new(
            request,
            callback,
            timer_id,
        );

        self.connected_clients.assign_request_to_client(&client_principal, request_id, request)?;
        Ok((client_principal, request_id))
    }

    fn next_request_id(&mut self) -> HttpRequestId {
        self.next_request_id += 1;
        self.next_request_id
    } 

    fn get_client_for_request(&self, request_id: HttpRequestId) -> Option<Principal> {
        let connected_clients_count = self.connected_clients.0.len();
        if connected_clients_count == 0 {
            return None;
        }
        let chosen_client_index = request_id as usize % connected_clients_count;
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

    pub fn report_http_failure(&mut self, client_principal: Principal, request_id: HttpRequestId, reason: HttpFailureReason) {
        self.connected_clients
            .0
            .get_mut(&client_principal)
            .and_then(|client| {
                client
                    .get_mut(&request_id)
                    .and_then(|r| {
                        r.failure_reason = Some(reason);
                        Some(r)
                    })
            });
    }

    pub fn handle_http_response(&mut self, client_principal: Principal, request_id: HttpRequestId, response: HttpResponse) -> Result<(), HttpFailureReason> {
        let client = self.connected_clients
            .0
            .get_mut(&client_principal)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "proxy not connected",
            )))?;
        let request = client
            .get_mut(&request_id)
            .ok_or(
            HttpFailureReason::RequestIdNotFound,
            )?;
        
        // a response arriving after an error has been reported is ignored
        if let Some(e) = request.failure_reason.clone() {
            return Err(e);
        }

        // a second response is ignored
        if request.response.is_none() {
            request.response = Some(response.clone());
    
    
            // response has been received, clear the timer if it was set
            if let Some(timer_id) = request.timer_id.take() {
                ic_cdk_timers::clear_timer(timer_id);
            }
    
            // if a callback was set, execute it
            if let Some(callback) = request.callback {
                ic_cdk::spawn(async move { callback(response).await });
            }
    
            // self.connected_clients.complete_request_for_client(client_principal, request_id)?;
        }

        Ok(())
    }

    pub fn get_http_request(&self, request_id: HttpRequestId) -> Option<HttpRequest> {
        for (_, requests) in
            self
                .connected_clients
                .0
                .iter() 
        {
            for (id, request) in requests {
                if id.to_owned() == request_id {
                    return Some(request.request.clone());
                }
            }
        }
        None
    }

    pub fn get_http_response(&self, request_id: HttpRequestId) -> GetHttpResponseResult {
        for (_, requests) in
            self
                .connected_clients
                .0
                .iter() 
        {
            for (id, request) in requests {
                if id.to_owned() == request_id {
                    return request.response
                        .as_ref()
                        .ok_or(
                        request.failure_reason
                            .clone()
                            .unwrap_or(HttpFailureReason::Unknown),
                        )
                        .cloned();
                }
            }
        }
        Err(HttpFailureReason::RequestIdNotFound)
    }
}


fn http_request_timeout(client_principal: Principal, request_id: HttpRequestId) {
    STATE.with(|state| {
        state.borrow_mut()
            .connected_clients
            .0
            .get_mut(&client_principal)
            .and_then(|client| {
                client
                    .get_mut(&request_id)
                    .and_then(|r| 
                    {
                        if r.response.is_none() {
                            r.failure_reason = Some(HttpFailureReason::RequestTimeout);
        
                            log(&format!(
                                "http_over_ws: HTTP request with id {} timed out",
                                request_id
                            ));
                        }
        
                        Some(r)
                    })
            });
        // if let Err(_) = state
        //     .borrow_mut()
        //     .connected_clients
        //     .complete_request_for_client(client_principal, request_id)
        // {
        //     log("cannot complete request");
        // }
    });
}



// #[cfg(test)]
// mod tests {
//     use candid::Principal;

//     use super::*;

//     #[test]
//     fn should_add_client_and_assign_request() {
//         let mut clients = ConnectedClients::new();
//         let client_principal = Principal::anonymous();
//         clients.add_client(client_principal);
//         assert_eq!(clients.0.len(), 1);

//         let request_id = 1;
//         assert!(clients.assign_request(request_id).is_ok());
//         assert!(clients
//             .0
//             .get(&client_principal)
//             .expect("client is not connected")
//             .contains(&request_id));
//     }

//     #[test]
//     fn should_not_assign_request() {
//         let mut clients = ConnectedClients::new();
//         assert!(clients.assign_request(1).is_err());
//     }

//     #[test]
//     fn should_complete_request() {
//         let mut clients = ConnectedClients::new();

//         let client_principal = Principal::anonymous();
//         clients.add_client(client_principal);
//         let request_id = 1;
//         assert!(clients.assign_request(request_id).is_ok());
//         assert!(clients
//             .complete_request_for_client(client_principal, request_id)
//             .is_ok());
//     }

//     #[test]
//     fn should_distribute_requests_among_clients() {
//         let mut clients = ConnectedClients::new();

//         let client_principal = Principal::from_text("aaaaa-aa").unwrap();
//         let another_client_principal = Principal::from_text("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();

//         clients.add_client(client_principal);
//         clients.add_client(another_client_principal);

//         let request_id = 1;
//         assert!(clients.assign_request(request_id).is_ok());

//         let request_id = 2;
//         assert!(clients.assign_request(request_id).is_ok());

//         let request_id = 3;
//         assert!(clients.assign_request(request_id).is_ok());

//         let request_id = 4;
//         assert!(clients.assign_request(request_id).is_ok());

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
