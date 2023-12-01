import IcWebSocket, { createWsConfig, generateRandomIdentity } from "ic-websocket-js";
import { ic_side_services_backend, canisterId } from "./src/canister/declarations/ic_side_services_backend";

const icNetworkUrl = process.env.IC_NETWORK_URL as string;
const gatewayUrl = process.env.IC_WS_GATEWAY_URL as string;

const wsConfig = createWsConfig({
  canisterId,
  canisterActor: ic_side_services_backend,
  networkUrl: icNetworkUrl,
  identity: generateRandomIdentity(),
});

console.log("Canister ID:", canisterId);

const ws = new IcWebSocket(gatewayUrl, {}, wsConfig);

ws.onopen = () => {
  console.log("WebSocket connected");
};

ws.onmessage = async (ev) => {
  const incomingMessage = ev.data;
  console.log("Message", incomingMessage);

  if ("HttpRequest" in incomingMessage) {
    const requestId = incomingMessage.HttpRequest[0];
    const request = incomingMessage.HttpRequest[1];

    console.log("Sending HTTP request to", request.url);
    // @ts-ignore
    const response = await fetch(request.url, {
      method: request.method,
      headers: request.headers,
      body: request.body.length > 0 ? new Uint8Array(request.body[0]!) : undefined,
    });

    console.log("Got response from", request.url, "Status:", response.status);

    ws.send({
      HttpResponse: [
        requestId,
        {
          status: BigInt(response.status),
          headers: Array.from(response.headers.entries()).map(([key, value]) => ({
            name: key,
            value,
          })),
          body: new Uint8Array(await response.arrayBuffer()),
        },
      ],
    });

    console.log("Sent response");
  } else if ("Error" in incomingMessage) {
    console.error("http-over-ws", incomingMessage.Error);
  }
};

ws.onclose = (ev) => {
  console.log("WebSocket disconnected", ev);
};
