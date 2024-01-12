use std::{collections::HashMap, time::Duration, cell::RefCell};
use candid::Principal;
use crate::{http_connection::{HttpFailureReason, HttpConnectionId, HttpCallback, HttpRequest, HttpRequestTimeoutMs, HttpResponse, GetHttpResponseResult, HttpConnection}, client_proxy::ClientProxy};


// local state
thread_local! {
    /* flexible */ pub static STATE: RefCell<State> = RefCell::new(State::new());
}

pub struct State {
    connected_proxies: ConnectedProxies,
    next_connection_id: HttpConnectionId,
}

impl State {
    pub fn new() -> Self {
        State {
            connected_proxies: ConnectedProxies::new(),
            next_connection_id: 0,
        }
    }

    pub fn add_proxy(&mut self, proxy_principal: Principal) {
        self.connected_proxies.add_proxy(proxy_principal);
    }


    pub fn remove_proxy(&mut self, proxy_principal: &Principal) -> Result<(), HttpFailureReason> {
        self.connected_proxies.remove_proxy(proxy_principal)
    }

    pub fn assign_connection(
        &mut self,
        request: HttpRequest,
        callback: Option<HttpCallback>,
        timeout_ms: Option<HttpRequestTimeoutMs>,
    ) -> Result<(Principal, HttpConnectionId), HttpFailureReason> {
        let connection_id = self.next_connection_id();

        let proxy_principal = self
            .get_proxy_for_connection(connection_id)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "no proxies connected",
            )))?
            .clone();

        let timer_id = timeout_ms.and_then(|millis| {
            Some(ic_cdk_timers::set_timer(
                Duration::from_millis(millis),
                move || {
                    http_connection_timeout(proxy_principal, connection_id);
                },
            ))
        });

        let connection = HttpConnection::new(
            connection_id,
            request,
            callback,
            timer_id,
        );

        self.connected_proxies.assign_connection_to_proxy(&proxy_principal, connection_id, connection)?;
        Ok((proxy_principal, connection_id))
    }

    fn next_connection_id(&mut self) -> HttpConnectionId {
        self.next_connection_id += 1;
        self.next_connection_id
    } 

    fn get_proxy_for_connection(&self, connection_id: HttpConnectionId) -> Option<Principal> {
        let connected_proxies_count = self.connected_proxies.0.len();
        if connected_proxies_count == 0 {
            return None;
        }
        let chosen_proxy_index = connection_id as usize % connected_proxies_count;
        // chosen_proxy_index is in [0, connected_proxies_count)
        // where connected_proxies_count is the number of proxies currently connected.
        // as no proxy is removed while executing this method,
        // the entry at 'chosen_proxy_index' is guaranteed to exist
        Some(
            self.connected_proxies
                .0
                .iter()
                .nth(chosen_proxy_index)
                .expect("proxy is not connected")
                .0
                .clone(),
        )
    }

    pub fn report_connection_failure(&mut self, proxy_principal: Principal, connection_id: HttpConnectionId, reason: HttpFailureReason) {
        self.connected_proxies
            .0
            .get_mut(&proxy_principal)
            .and_then(|proxy| {
                proxy.report_connection_failure(connection_id, reason);
                Some(proxy)
            });
    }

    pub fn handle_http_response(&mut self, proxy_principal: Principal, connection_id: HttpConnectionId, response: HttpResponse) -> Result<(), HttpFailureReason> {
        let proxy = self.connected_proxies
            .0
            .get_mut(&proxy_principal)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "proxy not connected",
            )))?;
        let connection = proxy.get_connection_mut(connection_id)?;
        
        connection.update_state(response)
    }

    pub fn get_http_connection(&self, connection_id: HttpConnectionId) -> Option<HttpRequest> {
        for (_, proxy) in
            self
                .connected_proxies
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
                .connected_proxies
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


fn http_connection_timeout(proxy_principal: Principal, connection_id: HttpConnectionId) {
    STATE.with(|state| {
        state.borrow_mut()
            .connected_proxies
            .0
            .get_mut(&proxy_principal)
            .and_then(|proxy| {
                let r = proxy.get_connection_mut(connection_id).and_then(|connection| {
                    connection.set_timeout();
                    Ok(connection)
                });
                Some(r)
            });
        // if let Err(_) = state
        //     .borrow_mut()
        //     .connected_proxies
        //     .complete_connection_for_proxy(proxy_principal, connection_id)
        // {
        //     log("cannot complete connection");
        // }
    });
}

pub(crate) struct ConnectedProxies(HashMap<Principal, ClientProxy>);

impl ConnectedProxies {
    fn new() -> Self {
        ConnectedProxies(HashMap::new())
    }

    fn add_proxy(&mut self, proxy_principal: Principal) {
        self.0.insert(proxy_principal, ClientProxy::new());
    }

    fn assign_connection_to_proxy(
        &mut self,
        proxy_principal: &Principal,
        connection_id: HttpConnectionId,
        connection: HttpConnection,
    ) -> Result<(), HttpFailureReason> {
        let proxy = self.0
            .get_mut(proxy_principal)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "proxy not connected",
            )))?;
        proxy.assign_connection(connection_id, connection);
        Ok(())
    }

    fn complete_connection_for_proxy(
        &mut self,
        proxy_principal: &Principal,
        connection_id: HttpConnectionId,
    ) -> Result<(), HttpFailureReason> {
        let proxy = self
            .0
            .get_mut(proxy_principal)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "proxy not connected",
            )))?;
        proxy.remove_connection(connection_id)?;
        Ok(())
    }

    fn remove_proxy(&mut self, proxy_principal: &Principal) -> Result<(), HttpFailureReason> {
        self.0
            .remove(proxy_principal)
            .ok_or(HttpFailureReason::ProxyError(String::from(
                "proxy not connected",
            )))?;
        Ok(())
    }
}