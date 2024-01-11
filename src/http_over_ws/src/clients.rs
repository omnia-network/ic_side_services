use std::collections::{HashMap, HashSet};

use candid::{CandidType, Deserialize, Principal};

use crate::types::{HttpFailureReason, HttpRequestId};

#[derive(CandidType, Clone, Deserialize)]
pub(crate) struct ConnectedClients(pub HashMap<Principal, HashSet<HttpRequestId>>);

impl ConnectedClients {
    pub fn new() -> Self {
        ConnectedClients(HashMap::new())
    }

    pub fn add_client(&mut self, client_principal: Principal) {
        self.0.insert(client_principal, HashSet::new());
    }

    fn get_client_for_request(&self, request_id: HttpRequestId) -> Option<Principal> {
        let connected_clients_count = self.0.len();
        if connected_clients_count == 0 {
            return None;
        }
        let chosen_client_index = request_id as usize % connected_clients_count;
        // chosen_client_index is in [0, connected_clients_count)
        // where connected_clients_count is the number of clients currently connected.
        // as no client is removed while executing this method,
        // the entry at 'chosen_client_index' is guaranteed to exist
        Some(
            self.0
                .iter()
                .nth(chosen_client_index)
                .expect("client is not connected")
                .0
                .clone(),
        )
    }

    fn assign_request_to_client(
        &mut self,
        client_principal: &Principal,
        request_id: HttpRequestId,
    ) -> Result<(), HttpFailureReason> {
        self.0
            .get_mut(&client_principal)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "proxy not connected",
            )))?
            .insert(request_id);
        Ok(())
    }

    pub fn assign_request(
        &mut self,
        request_id: HttpRequestId,
    ) -> Result<Principal, HttpFailureReason> {
        let client_principal = self
            .get_client_for_request(request_id)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "no clients connected",
            )))?
            .clone();
        self.assign_request_to_client(&client_principal, request_id)?;
        Ok(client_principal)
    }

    pub fn complete_request_for_client(
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
            .then(|| ())
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "client has not been assigned the request",
            )))?;
        Ok(())
    }

    pub fn remove_client(&mut self, client_principal: &Principal) -> Result<(), HttpFailureReason> {
        self.0
            .remove(client_principal)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "client not connected",
            )))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use candid::Principal;

    use super::*;

    #[test]
    fn should_add_client_and_assign_request() {
        let mut clients = ConnectedClients::new();
        let client_principal = Principal::anonymous();
        clients.add_client(client_principal);
        assert_eq!(clients.0.len(), 1);

        let request_id = 1;
        assert!(clients.assign_request(request_id).is_ok());
        assert!(clients
            .0
            .get(&client_principal)
            .expect("client is not connected")
            .contains(&request_id));
    }

    #[test]
    fn should_not_assign_request() {
        let mut clients = ConnectedClients::new();
        assert!(clients.assign_request(1).is_err());
    }

    #[test]
    fn should_complete_request() {
        let mut clients = ConnectedClients::new();

        let client_principal = Principal::anonymous();
        clients.add_client(client_principal);
        let request_id = 1;
        assert!(clients.assign_request(request_id).is_ok());
        assert!(clients
            .complete_request_for_client(client_principal, request_id)
            .is_ok());
    }

    #[test]
    fn should_distribute_requests_among_clients() {
        let mut clients = ConnectedClients::new();

        let client_principal = Principal::from_text("aaaaa-aa").unwrap();
        let another_client_principal = Principal::from_text("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();

        clients.add_client(client_principal);
        clients.add_client(another_client_principal);

        let request_id = 1;
        assert!(clients.assign_request(request_id).is_ok());

        let request_id = 2;
        assert!(clients.assign_request(request_id).is_ok());

        let request_id = 3;
        assert!(clients.assign_request(request_id).is_ok());

        let request_id = 4;
        assert!(clients.assign_request(request_id).is_ok());

        assert!(
            clients
                .0
                .get(&client_principal)
                .expect("client is not connected")
                .len()
                == 2
        );
        assert!(
            clients
                .0
                .get(&another_client_principal)
                .expect("client is not connected")
                .len()
                == 2
        );
    }
}
