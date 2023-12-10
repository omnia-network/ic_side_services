export const idlFactory = ({ IDL }) => {
  const FluxNetwork = IDL.Variant({
    'mainnet' : IDL.Null,
    'local' : IDL.Null,
    'testnet' : IDL.Null,
  });
  const HttpRequestId = IDL.Nat32;
  const ConnectedClients = IDL.Record({
    'busy_clients' : IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Vec(HttpRequestId))),
    'idle_clients' : IDL.Vec(IDL.Principal),
  });
  const HttpMethod = IDL.Variant({
    'GET' : IDL.Null,
    'PUT' : IDL.Null,
    'DELETE' : IDL.Null,
    'HEAD' : IDL.Null,
    'POST' : IDL.Null,
  });
  const HttpHeader = IDL.Record({ 'value' : IDL.Text, 'name' : IDL.Text });
  const PrettyHttpRequest = IDL.Record({
    'url' : IDL.Text,
    'method' : HttpMethod,
    'body' : IDL.Opt(IDL.Text),
    'headers' : IDL.Vec(HttpHeader),
  });
  const PrettyHttpResponse = IDL.Record({
    'status' : IDL.Nat,
    'body' : IDL.Text,
    'headers' : IDL.Vec(HttpHeader),
  });
  const HttpRequestFailureReason = IDL.Variant({
    'ErrorFromClient' : IDL.Text,
    'NotFound' : IDL.Null,
    'Timeout' : IDL.Null,
    'Unknown' : IDL.Null,
  });
  const GetHttpResponseResult = IDL.Variant({
    'Ok' : PrettyHttpResponse,
    'Err' : HttpRequestFailureReason,
  });
  const ClientPrincipal = IDL.Principal;
  const ClientKey = IDL.Record({
    'client_principal' : ClientPrincipal,
    'client_nonce' : IDL.Nat64,
  });
  const CanisterWsCloseArguments = IDL.Record({ 'client_key' : ClientKey });
  const CanisterWsCloseResult = IDL.Variant({
    'Ok' : IDL.Null,
    'Err' : IDL.Text,
  });
  const CanisterWsGetMessagesArguments = IDL.Record({ 'nonce' : IDL.Nat64 });
  const CanisterOutputMessage = IDL.Record({
    'key' : IDL.Text,
    'content' : IDL.Vec(IDL.Nat8),
    'client_key' : ClientKey,
  });
  const CanisterOutputCertifiedMessages = IDL.Record({
    'messages' : IDL.Vec(CanisterOutputMessage),
    'cert' : IDL.Vec(IDL.Nat8),
    'tree' : IDL.Vec(IDL.Nat8),
    'is_end_of_queue' : IDL.Bool,
  });
  const CanisterWsGetMessagesResult = IDL.Variant({
    'Ok' : CanisterOutputCertifiedMessages,
    'Err' : IDL.Text,
  });
  const WebsocketMessage = IDL.Record({
    'sequence_num' : IDL.Nat64,
    'content' : IDL.Vec(IDL.Nat8),
    'client_key' : ClientKey,
    'timestamp' : IDL.Nat64,
    'is_service_message' : IDL.Bool,
  });
  const CanisterWsMessageArguments = IDL.Record({ 'msg' : WebsocketMessage });
  const HttpRequest = IDL.Record({
    'url' : IDL.Text,
    'method' : HttpMethod,
    'body' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'headers' : IDL.Vec(HttpHeader),
  });
  const HttpResponse = IDL.Record({
    'status' : IDL.Nat,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HttpHeader),
  });
  const HttpOverWsMessage = IDL.Variant({
    'Error' : IDL.Tuple(IDL.Opt(HttpRequestId), IDL.Text),
    'HttpRequest' : IDL.Tuple(HttpRequestId, HttpRequest),
    'HttpResponse' : IDL.Tuple(HttpRequestId, HttpResponse),
  });
  const CanisterWsMessageResult = IDL.Variant({
    'Ok' : IDL.Null,
    'Err' : IDL.Text,
  });
  const GatewayPrincipal = IDL.Principal;
  const CanisterWsOpenArguments = IDL.Record({
    'gateway_principal' : GatewayPrincipal,
    'client_nonce' : IDL.Nat64,
  });
  const CanisterWsOpenResult = IDL.Variant({
    'Ok' : IDL.Null,
    'Err' : IDL.Text,
  });
  return IDL.Service({
    'flux_fetch_balance' : IDL.Func([], [], []),
    'flux_get_balance' : IDL.Func([], [IDL.Opt(IDL.Int32)], ['query']),
    'flux_login' : IDL.Func([], [], []),
    'flux_logout' : IDL.Func([], [], []),
    'get_addresses' : IDL.Func([], [IDL.Text, IDL.Text], ['query']),
    'get_connected_clients' : IDL.Func([], [ConnectedClients], ['query']),
    'get_http_request' : IDL.Func(
        [HttpRequestId],
        [IDL.Opt(PrettyHttpRequest)],
        ['query'],
      ),
    'get_http_response' : IDL.Func(
        [HttpRequestId],
        [GetHttpResponseResult],
        ['query'],
      ),
    'get_logs' : IDL.Func(
        [],
        [IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text))],
        ['query'],
      ),
    'set_canister_public_key' : IDL.Func([IDL.Opt(IDL.Text)], [], []),
    'sign_with_ecdsa' : IDL.Func([IDL.Text, IDL.Opt(IDL.Text)], [IDL.Text], []),
    'ws_close' : IDL.Func(
        [CanisterWsCloseArguments],
        [CanisterWsCloseResult],
        [],
      ),
    'ws_get_messages' : IDL.Func(
        [CanisterWsGetMessagesArguments],
        [CanisterWsGetMessagesResult],
        ['query'],
      ),
    'ws_message' : IDL.Func(
        [CanisterWsMessageArguments, IDL.Opt(HttpOverWsMessage)],
        [CanisterWsMessageResult],
        [],
      ),
    'ws_open' : IDL.Func([CanisterWsOpenArguments], [CanisterWsOpenResult], []),
  });
};
export const init = ({ IDL }) => {
  const FluxNetwork = IDL.Variant({
    'mainnet' : IDL.Null,
    'local' : IDL.Null,
    'testnet' : IDL.Null,
  });
  return [FluxNetwork];
};
