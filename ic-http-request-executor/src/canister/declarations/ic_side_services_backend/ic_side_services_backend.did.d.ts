import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export type BitcoinAddress = string;
export interface CanisterOutputCertifiedMessages {
  'messages' : Array<CanisterOutputMessage>,
  'cert' : Uint8Array | number[],
  'tree' : Uint8Array | number[],
  'is_end_of_queue' : boolean,
}
export interface CanisterOutputMessage {
  'key' : string,
  'content' : Uint8Array | number[],
  'client_key' : ClientKey,
}
export interface CanisterWsCloseArguments { 'client_key' : ClientKey }
export type CanisterWsCloseResult = { 'Ok' : null } |
  { 'Err' : string };
export interface CanisterWsGetMessagesArguments { 'nonce' : bigint }
export type CanisterWsGetMessagesResult = {
    'Ok' : CanisterOutputCertifiedMessages
  } |
  { 'Err' : string };
export interface CanisterWsMessageArguments { 'msg' : WebsocketMessage }
export type CanisterWsMessageResult = { 'Ok' : null } |
  { 'Err' : string };
export interface CanisterWsOpenArguments {
  'gateway_principal' : GatewayPrincipal,
  'client_nonce' : bigint,
}
export type CanisterWsOpenResult = { 'Ok' : null } |
  { 'Err' : string };
export interface ClientKey {
  'client_principal' : ClientPrincipal,
  'client_nonce' : bigint,
}
export type ClientPrincipal = Principal;
export interface ConnectedClients {
  'busy_clients' : Array<[Principal, Uint32Array | number[]]>,
  'idle_clients' : Array<Principal>,
}
export type FluxNetwork = { 'mainnet' : null } |
  { 'local' : null } |
  { 'testnet' : null };
export type GatewayPrincipal = Principal;
export type GetHttpResponseResult = { 'Ok' : PrettyHttpResponse } |
  { 'Err' : HttpRequestFailureReason };
export interface HttpHeader { 'value' : string, 'name' : string }
export type HttpMethod = { 'GET' : null } |
  { 'PUT' : null } |
  { 'DELETE' : null } |
  { 'HEAD' : null } |
  { 'POST' : null };
export type HttpOverWsMessage = { 'Error' : [[] | [HttpRequestId], string] } |
  { 'HttpRequest' : [HttpRequestId, HttpRequest] } |
  { 'HttpResponse' : [HttpRequestId, HttpResponse] };
export interface HttpRequest {
  'url' : string,
  'method' : HttpMethod,
  'body' : [] | [Uint8Array | number[]],
  'headers' : Array<HttpHeader>,
}
export type HttpRequestFailureReason = { 'ErrorFromClient' : string } |
  { 'NotFound' : null } |
  { 'Timeout' : null } |
  { 'Unknown' : null };
export type HttpRequestId = number;
export interface HttpResponse {
  'status' : bigint,
  'body' : Uint8Array | number[],
  'headers' : Array<HttpHeader>,
}
export interface PrettyHttpRequest {
  'url' : string,
  'method' : HttpMethod,
  'body' : [] | [string],
  'headers' : Array<HttpHeader>,
}
export interface PrettyHttpResponse {
  'status' : bigint,
  'body' : string,
  'headers' : Array<HttpHeader>,
}
export interface WebsocketMessage {
  'sequence_num' : bigint,
  'content' : Uint8Array | number[],
  'client_key' : ClientKey,
  'timestamp' : bigint,
  'is_service_message' : boolean,
}
export interface _SERVICE {
  'flux_fetch_balance' : ActorMethod<[], undefined>,
  'flux_get_balance' : ActorMethod<[], [] | [number]>,
  'flux_login' : ActorMethod<[], undefined>,
  'flux_logout' : ActorMethod<[], undefined>,
  'get_addresses' : ActorMethod<[], [string, string]>,
  'get_connected_clients' : ActorMethod<[], ConnectedClients>,
  'get_http_request' : ActorMethod<[HttpRequestId], [] | [PrettyHttpRequest]>,
  'get_http_response' : ActorMethod<[HttpRequestId], GetHttpResponseResult>,
  'get_logs' : ActorMethod<[], Array<[string, string]>>,
  'set_canister_public_key' : ActorMethod<[[] | [string]], undefined>,
  'sign_with_ecdsa' : ActorMethod<[string, [] | [string]], string>,
  'ws_close' : ActorMethod<[CanisterWsCloseArguments], CanisterWsCloseResult>,
  'ws_get_messages' : ActorMethod<
    [CanisterWsGetMessagesArguments],
    CanisterWsGetMessagesResult
  >,
  'ws_message' : ActorMethod<
    [CanisterWsMessageArguments, [] | [HttpOverWsMessage]],
    CanisterWsMessageResult
  >,
  'ws_open' : ActorMethod<[CanisterWsOpenArguments], CanisterWsOpenResult>,
}
