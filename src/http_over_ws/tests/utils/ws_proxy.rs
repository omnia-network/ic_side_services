use candid::Principal;
use http_over_ws::HttpOverWsMessage;

use super::{
    actor::CanisterMethod, ic_env::TestEnv, identity::generate_random_principal,
    rand::generate_random_nonce,
};
use ic_websocket_cdk::{
    CanisterWsCloseArguments, CanisterWsCloseResult, CanisterWsGetMessagesArguments,
    CanisterWsGetMessagesResult, CanisterWsMessageArguments, CanisterWsMessageResult,
    CanisterWsOpenArguments, CanisterWsOpenResult, ClientKey, WebsocketMessage,
};

type WsMessageContentBytes = Vec<u8>;

pub struct WsProxyClient<'a> {
    test_env: &'a TestEnv,
    client_key: ClientKey,
    gateway_principal: Principal,
    outgoing_messages_sequence_num: u64,
    polling_nonce: u64,
}

impl<'a> WsProxyClient<'a> {
    pub fn new(test_env: &'a TestEnv) -> Self {
        Self {
            test_env,
            client_key: generate_random_client_key(),
            gateway_principal: generate_random_principal(),
            outgoing_messages_sequence_num: 0,
            polling_nonce: 0,
        }
    }

    pub fn open_ws_connection(&self) {
        let res: CanisterWsOpenResult = self.test_env.call_canister_method_with_panic(
            self.client_key.client_principal,
            CanisterMethod::WsOpen,
            CanisterWsOpenArguments::new(self.client_key.client_nonce, self.gateway_principal),
        );

        assert!(res.is_ok());
    }

    pub fn send_ws_message(&mut self, message: WsMessageContentBytes) {
        self.outgoing_messages_sequence_num += 1;

        let res: CanisterWsMessageResult = self.test_env.call_canister_method_with_panic(
            self.client_key.client_principal,
            CanisterMethod::WsMessage,
            CanisterWsMessageArguments::new(WebsocketMessage::new(
                self.client_key.clone(),
                self.outgoing_messages_sequence_num,
                0, // we don't need a timestamp here
                false,
                message,
            )),
        );

        assert!(res.is_ok());
    }

    pub fn get_ws_messages(&mut self) -> Vec<WsMessageContentBytes> {
        let res: CanisterWsGetMessagesResult = self.test_env.call_canister_method_with_panic(
            self.client_key.client_principal,
            CanisterMethod::WsGetMessages,
            CanisterWsGetMessagesArguments::new(self.polling_nonce),
        );

        match res {
            CanisterWsGetMessagesResult::Ok(messages) => {
                self.polling_nonce += messages.messages.len() as u64;

                messages
                    .messages
                    .into_iter()
                    .filter_map(|m| {
                        // TODO: deserialize and check message
                        Some(m.content)
                    })
                    .collect()
            }
            CanisterWsGetMessagesResult::Err(_) => panic!("Failed to get messages"),
        }
    }

    pub fn close_ws_connection(&self) {
        let res: CanisterWsCloseResult = self.test_env.call_canister_method_with_panic(
            self.gateway_principal,
            CanisterMethod::WsClose,
            CanisterWsCloseArguments::new(self.client_key.clone()),
        );

        assert!(res.is_ok());
    }
}

fn generate_random_client_key() -> ClientKey {
    ClientKey::new(generate_random_principal(), generate_random_nonce())
}
